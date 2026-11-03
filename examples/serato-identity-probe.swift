// Temporary CoreMIDI identity probe. Capture mode never sends MIDI.
import CoreMIDI
import Darwin
import Foundation

enum ProbeError: Error {
    case midi(String, OSStatus)
    case arguments(String)
    case mismatch(String)
    case timeout(String)
}

func check(_ status: OSStatus, _ operation: String) throws {
    guard status == noErr else { throw ProbeError.midi(operation, status) }
}

func emit(_ event: String, _ fields: [String: String] = [:]) {
    let values = fields.sorted { $0.key < $1.key }
        .map { "\($0.key)=\(String(reflecting: $0.value))" }.joined(separator: " ")
    let line = "\(Date().timeIntervalSince1970) \(event) \(values)\n"
    FileHandle.standardOutput.write(Data(line.utf8))
}

struct Options {
    var name = "DJControl Starlight"
    var seconds: TimeInterval = 45
    var selfTest = false

    init() throws {
        var args = CommandLine.arguments.dropFirst().makeIterator()
        while let arg = args.next() {
            switch arg {
            case "--self-test":
                selfTest = true
            case "--name":
                guard let value = args.next(), !value.isEmpty else {
                    throw ProbeError.arguments("--name requires a nonempty endpoint name")
                }
                name = value
            case "--seconds":
                guard let value = args.next(), let duration = Double(value),
                      duration.isFinite, duration >= 1, duration <= 120 else {
                    throw ProbeError.arguments("--seconds requires a number from 1 through 120")
                }
                seconds = duration
            case "--help":
                print("""
                serato-identity-probe [--name NAME] [--seconds 1..120] [--self-test]
                Default: DJControl Starlight, Hercules metadata, 45 seconds, receive only.
                --self-test uses unique ports and sends only to its own endpoints.
                No HID, audio routing, USB impersonation or authentication replies.
                """)
                exit(EXIT_SUCCESS)
            default:
                throw ProbeError.arguments("unknown argument: \(arg)")
            }
        }
    }
}

// All mutable capture state is protected by lock; CoreMIDI calls off the main thread.
final class Capture: @unchecked Sendable {
    private let lock = NSLock()
    private let changed = DispatchSemaphore(value: 0)
    private var bytes: [UInt8] = []
    private var packets = 0
    let direction: String

    init(_ direction: String) { self.direction = direction }

    func receive(_ list: UnsafePointer<MIDIPacketList>) {
        for packet in list.unsafeSequence() {
            let data = Array(packet.bytes())
            lock.withLock {
                bytes.append(contentsOf: data)
                packets += 1
                emit("MIDI_RECEIVED", [
                    "direction": direction,
                    "timestamp": String(packet.pointee.timeStamp),
                    "bytes": data.map { String(format: "%02x", $0) }.joined(separator: " "),
                ])
            }
            changed.signal()
        }
    }

    func totals() -> (packets: Int, bytes: Int) {
        lock.withLock { (packets, bytes.count) }
    }

    func wait(for expected: [UInt8]) throws {
        let deadline = DispatchTime.now() + 3
        while true {
            let received = lock.withLock { bytes }
            if received.count >= expected.count {
                guard received == expected else { throw ProbeError.mismatch(direction) }
                return
            }
            guard changed.wait(timeout: deadline) == .success else {
                throw ProbeError.timeout(direction)
            }
        }
    }
}

func stringProperty(_ object: MIDIObjectRef, _ key: CFString) throws -> String {
    var value: Unmanaged<CFString>?
    try check(MIDIObjectGetStringProperty(object, key, &value), "read \(key)")
    guard let value else { throw ProbeError.mismatch("missing property \(key)") }
    return value.takeRetainedValue() as String
}

func describe(_ endpoint: MIDIEndpointRef, _ direction: String) throws {
    var entity: MIDIEntityRef = 0
    var uniqueID: Int32 = 0
    // A virtual endpoint has no owning entity; macOS reports object-not-found.
    let entityStatus = MIDIEndpointGetEntity(endpoint, &entity)
    if entityStatus != kMIDIObjectNotFound {
        try check(entityStatus, "get endpoint entity")
    }
    try check(MIDIObjectGetIntegerProperty(endpoint, kMIDIPropertyUniqueID, &uniqueID), "get unique ID")
    emit("IDENTITY_ENDPOINT", [
        "direction": direction, "endpoint": String(endpoint), "entity": String(entity),
        "entity_status": String(entityStatus),
        "unique_id": String(uniqueID),
        "name": try stringProperty(endpoint, kMIDIPropertyName),
        "manufacturer": try stringProperty(endpoint, kMIDIPropertyManufacturer),
        "model": try stringProperty(endpoint, kMIDIPropertyModel),
    ])
}

func selfTest(_ client: MIDIClientRef, _ endpoints: (source: MIDIEndpointRef, destination: MIDIEndpointRef),
              _ destinationCapture: Capture) throws {
    let sourceCapture = Capture("source-loopback")
    var input: MIDIPortRef = 0
    var output: MIDIPortRef = 0
    try check(MIDIInputPortCreateWithBlock(client, "Loopback input" as CFString, &input) {
        list, _ in sourceCapture.receive(list)
    }, "create self-test input")
    try check(MIDIPortConnectSource(input, endpoints.source, nil), "subscribe self-test source")
    try check(MIDIOutputPortCreate(client, "Loopback output" as CFString, &output), "create self-test output")
    let sysex: [UInt8] = [0xf0, 0x7d] + (0..<600).map { UInt8($0 % 128) } + [0xf7]
    let channel: [UInt8] = [0xb0, 0x10, 0x40, 0x90, 0x07, 0x01]
    let list = MIDIPacketList.Builder(byteSize: 2048)
    for data in [sysex, channel] {
        guard list.append(timestamp: mach_absolute_time(), data: data) != nil else {
            throw ProbeError.mismatch("self-test packet-list capacity")
        }
    }
    // Both real receive paths are subscribed before either transmission.
    let statuses = list.withUnsafePointer { packets in
        (MIDISend(output, endpoints.destination, packets), MIDIReceived(endpoints.source, packets))
    }
    try check(statuses.0, "send self-test destination")
    try check(statuses.1, "publish self-test source")
    let expected = sysex + channel
    try destinationCapture.wait(for: expected)
    try sourceCapture.wait(for: expected)
    emit("SERATO_IDENTITY_LOOPBACK_PASS", [
        "bytes_per_direction": String(expected.count), "directions": "2",
        "sysex_bytes": String(sysex.count), "physical_ports_opened": "0",
    ])
}

func run() throws {
    let options = try Options()
    let name = options.selfTest ? "Serato Identity Loopback \(UUID().uuidString)" : options.name
    // Never shadow a real controller or another running instance.
    for (count, endpoint) in [(MIDIGetNumberOfSources(), MIDIGetSource),
                               (MIDIGetNumberOfDestinations(), MIDIGetDestination)] {
        for index in 0..<count {
            if try stringProperty(endpoint(index), kMIDIPropertyName) == name {
                throw ProbeError.arguments("an endpoint already uses \(name); leave it untouched")
            }
        }
    }
    let capture = Capture("destination")
    var client: MIDIClientRef = 0
    try check(MIDIClientCreateWithBlock(name as CFString, &client, nil), "create MIDI client")
    defer {
        let status = MIDIClientDispose(client)
        let totals = capture.totals()
        emit(status == noErr ? "SERATO_IDENTITY_STOPPED" : "SERATO_IDENTITY_CLEANUP_ERROR", [
            "name": name, "client_dispose_status": String(status),
            "received_packets": String(totals.packets), "received_bytes": String(totals.bytes),
        ])
        if status != noErr { exit(EXIT_FAILURE) }
    }
    var destination: MIDIEndpointRef = 0
    var source: MIDIEndpointRef = 0
    try check(MIDIDestinationCreateWithBlock(client, name as CFString, &destination) {
        list, _ in capture.receive(list)
    }, "create return destination")
    try check(MIDIObjectSetStringProperty(destination, kMIDIPropertyManufacturer, "Hercules" as CFString),
              "set destination manufacturer")
    try check(MIDIObjectSetStringProperty(destination, kMIDIPropertyModel, "DJControl Starlight" as CFString),
              "set destination model")
    try check(MIDISourceCreate(client, name as CFString, &source), "publish source")
    try check(MIDIObjectSetStringProperty(source, kMIDIPropertyManufacturer, "Hercules" as CFString),
              "set source manufacturer")
    try check(MIDIObjectSetStringProperty(source, kMIDIPropertyModel, "DJControl Starlight" as CFString),
              "set source model")
    try describe(destination, "destination")
    try describe(source, "source")
    if options.selfTest {
        try selfTest(client, (source, destination), capture)
        return
    }
    let interrupt = DispatchSource.makeSignalSource(signal: SIGINT, queue: .main)
    signal(SIGINT, SIG_IGN)
    interrupt.setEventHandler { CFRunLoopStop(CFRunLoopGetMain()) }
    interrupt.resume()
    defer {
        interrupt.cancel()
        signal(SIGINT, SIG_DFL)
    }
    let timer = Timer.scheduledTimer(withTimeInterval: options.seconds, repeats: false) { _ in
        CFRunLoopStop(CFRunLoopGetMain())
    }
    defer { timer.invalidate() }
    emit("SERATO_IDENTITY_READY", [
        "name": name, "seconds": String(options.seconds), "mode": "receive-only",
        "note": "no USB/audio device or driver entity; no authentication replies",
    ])
    CFRunLoopRun()
}

do {
    try run()
} catch {
    emit("SERATO_IDENTITY_ERROR", ["error": String(describing: error)])
    exit(EXIT_FAILURE)
}
