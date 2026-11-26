using System;
using System.IO;

namespace S4UsbCapture
{
    public sealed class Summary
    {
        public bool FileComplete { get; internal set; }
        public long PacketCount { get; internal set; }
        public long TargetPackets { get; internal set; }
        public long TargetOutRequests { get; internal set; }
        public long MotorRequests { get; internal set; }
        public long RingRequests { get; internal set; }
        public bool HasMotorAndRingCoverage { get { return MotorRequests > 0 && RingRequests > 0; } }
        public uint Snaplen { get; internal set; }
        public long Bytes { get; internal set; }
        public bool UsbPacketLossKnown { get { return false; } }
    }

    public static class UsbPcapReader
    {
        public static Summary Read(string path, int deviceAddress)
        {
            if (deviceAddress < 0 || deviceAddress > UInt16.MaxValue)
                throw new ArgumentOutOfRangeException("deviceAddress");

            using (FileStream stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read))
            using (BinaryReader reader = new BinaryReader(stream))
            {
                Summary result = new Summary();
                result.Bytes = stream.Length;
                if (stream.Length < 24) throw new InvalidDataException("Incomplete PCAP global header.");
                if (reader.ReadUInt32() != 0xa1b2c3d4U)
                    throw new InvalidDataException("Expected little-endian classic PCAP magic.");
                if (reader.ReadUInt16() != 2 || reader.ReadUInt16() != 4)
                    throw new InvalidDataException("Expected PCAP version 2.4.");
                reader.ReadUInt32();
                reader.ReadUInt32();
                result.Snaplen = reader.ReadUInt32();
                if (result.Snaplen == 0) throw new InvalidDataException("Invalid zero snaplen.");
                if (reader.ReadUInt32() != 249) throw new InvalidDataException("Expected USBPcap linktype 249.");

                while (stream.Position < stream.Length)
                {
                    if (stream.Length - stream.Position < 16) throw new InvalidDataException("Incomplete PCAP record header.");
                    reader.ReadUInt32();
                    uint usec = reader.ReadUInt32();
                    uint captured = reader.ReadUInt32();
                    uint original = reader.ReadUInt32();
                    if (usec >= 1000000) throw new InvalidDataException("Invalid PCAP timestamp.");
                    if (captured > original || captured > result.Snaplen)
                        throw new InvalidDataException("Invalid PCAP captured length.");
                    if (captured != original) throw new InvalidDataException("Snap-truncated PCAP packet.");
                    if (captured > stream.Length - stream.Position) throw new InvalidDataException("Incomplete PCAP packet.");
                    if (captured < 27) throw new InvalidDataException("Incomplete USBPcap header.");
                    long start = stream.Position;
                    ushort headerLength = reader.ReadUInt16();
                    reader.ReadUInt64();
                    reader.ReadUInt32();
                    reader.ReadUInt16();
                    byte info = reader.ReadByte();
                    reader.ReadUInt16();
                    ushort device = reader.ReadUInt16();
                    byte endpoint = reader.ReadByte();
                    byte transfer = reader.ReadByte();
                    uint dataLength = reader.ReadUInt32();
                    if (headerLength < 27 || headerLength > captured || dataLength != captured - headerLength)
                        throw new InvalidDataException("Invalid USBPcap header or data length.");
                    if (transfer == 2 && headerLength < 28)
                        throw new InvalidDataException("Incomplete USBPcap control header.");
                    if (transfer == 0)
                    {
                        if (headerLength < 39) throw new InvalidDataException("Incomplete USBPcap isochronous header.");
                        reader.ReadUInt32();
                        uint packets = reader.ReadUInt32();
                        if (39UL + 12UL * packets > headerLength)
                            throw new InvalidDataException("Incomplete USBPcap isochronous packet descriptors.");
                    }
                    result.PacketCount++;
                    if (device == deviceAddress)
                    {
                        result.TargetPackets++;
                        if (endpoint == 0x02 && (info & 1) == 0)
                        {
                            result.TargetOutRequests++;
                            // HID OUT payload is directly available on bulk/interrupt transfers.
                            if ((transfer == 1 || transfer == 3) && dataLength > 0 && dataLength <= 95)
                            {
                                stream.Position = start + headerLength;
                                byte report = reader.ReadByte();
                                if (report == 0x31 && (dataLength == 11 || dataLength == 95)) result.MotorRequests++;
                                if (report == 0x32 && (dataLength == 41 || dataLength == 95)) result.RingRequests++;
                            }
                        }
                    }
                    stream.Position = start + captured;
                }
                result.FileComplete = true;
                return result;
            }
        }
    }
}
