# Standalone hardware bridge research

Recorded October 8, 2026, from the October 7-8 hardware discussion and source
checks. This is a pre-purchase research record, not an approved implementation
plan or a hardware validation report.

No board was selected or purchased during the discussion. No standalone audio
bridge was built or measured. The current macOS app still leaves audio directly
connected between Djay and the S4.

## Requirements and budget

- Move HID-to-MIDI translation and returning LED feedback onto the bridge.
- Preserve S4 audio inputs as well as master/headphone playback. The research
  baseline is at least eight capture channels plus four playback channels,
  simultaneously; the complete USB descriptors must establish the final profile.
- Provide independent, simultaneous USB host and device connections: host toward
  the S4, device toward the Mac.
- Present class-compliant USB audio and bidirectional MIDI together to the Mac.
  A custom Djay mapping remains acceptable and still needs installation.
- Keep added audio latency low and avoid underruns, overruns, and long-run drift.
  No numeric acceptance limit or soak-test duration has been agreed.
- Keep Rust viable. A Teensy implementation does not inherently require C++.
- Leave screens, motors, and controller impersonation deferred.

The initial approximate budget was USD 30; the user subsequently explored a
USD 50 limit. Whether this is board-only or the complete assembly remains open.
Listed prices below exclude tax and shipping. Storage, cables, power, host
adapters, cooling where needed, and an enclosure must be costed separately.

The earlier output-only scope in [design.md](design.md#5-audio-and-usb-topology)
does not describe the corrected hardware requirement.

## USB topology and audio channels

```text
Mac / Djay
    |
    | USB composite device: audio capture/playback + MIDI
    |
Hardware bridge
    |
    | USB host: S4 audio capture/playback + HID controls/LEDs
    |
S4 MK3, powered by its own adapter
    +-- line/phono/microphone inputs
    +-- master / booth / headphones
```

Multiple interfaces can share one USB device connection. The Rust
[`usb-device` stack][usb-device] explicitly supports composite devices.
Interfaces are not separate addressed USB devices; separate devices sharing a
cable normally require a hub. An ordinary hub cannot let the Mac own the S4's
audio while another host independently owns its HID interface.

[NI's microphone guide][ni-inputs] identifies MIC D as USB input channels 7/8,
shared with line input D. MIC C similarly shares input C. The microphone sockets
therefore do not imply extra independently available USB capture pairs.
[Mixxx documents][mixxx] playback channels 1/2 as main and 3/4 as headphones.
Complete channel maps, source switching, formats, and Djay input routing still
need verification on the actual S4.

For illustration, eight capture plus four playback channels at packed 24-bit
resolution require:

| Sample rate | Combined PCM payload, before USB overhead |
| --- | ---: |
| 48 kHz | 13.824 Mbit/s |
| 96 kHz | 27.648 Mbit/s |

These are calculations, not verified S4 endpoint formats. Four-byte sample
containers would require more bandwidth. Since even the 48 kHz example exceeds
12 Mbit/s, independent 480 Mbit/s High-Speed links on both sides are the prudent
selection criterion. Two connectors or an "HS" controller name alone do not
establish that capability on a particular board.

## Latency, clocking, and RAM

USB bandwidth and CPU speed do not establish end-to-end latency. The bridge adds
receive queues, audio buffers, and outgoing USB scheduling in both directions.
Input monitoring through software also traverses the capture and playback paths.

The S4 and Mac-facing gadget are separate clock domains. A finite buffer alone
does not solve clock drift: the implementation must coordinate feedback/pacing
or use suitable rate adaptation. No bit-perfect or dropout-free result was
established. The assistant suggested 2-3 ms additional latency as a provisional
engineering target; the user did not adopt it as an acceptance requirement.

At 96 kHz, twelve channels using four-byte sample containers occupy:

| Audio held across all channels | Memory for one buffer set |
| --- | ---: |
| 10 ms | 45 KiB |
| 20 ms | 90 KiB |
| 100 ms | 450 KiB |

There will be multiple buffers. These durations illustrate memory capacity,
not recommended buffering or predicted latency. A minimal headless Linux bridge
is a plausible fit for Lyra's 128 MB RAM; actual free memory and peak use remain
unmeasured. Linux, dependencies, and other services are more significant than
the PCM buffer footprint. More RAM does not inherently lower latency.

Lyra B's 256 MB NAND is storage, not additional RAM. A 1 GB Pi has ample estimated
memory headroom for this workload without needing a desktop environment.

## Software paths

### Linux

Linux provides configurable [UAC2 and MIDI gadget functions][linux-gadgets],
including capture/playback channel masks, sample rates, and sample sizes.
The gadget creates an audio interface; it does not automatically forward audio
to the S4. Host audio drivers, forwarding, and clock coordination are separate.

[Current `u_audio.c`][linux-audio] provides capture/playback pitch controls that
can affect feedback and packet pacing. Their presence depends on the selected
kernel and does not itself implement synchronization with a downstream sound
card. [`alsaloop`][alsaloop] documents buffering, requested latency, and adaptive
clock synchronization/resampling, but its existence is not an S4 performance
result.

Linux is the leading prototype path because it supplies existing audio drivers
and permits reuse of desktop Rust code. The exact board image must expose a USB
device controller and include UAC2, MIDI gadget, and USB-audio host support.

### Embedded Rust and Teensy

[`teensy4-bsp`][teensy-rust] supports Teensy 4.0 and 4.1 and exposes
[`imxrt-usbd`][imxrt-usbd], a full/high-speed device-side USB driver.
This corrects the initial suggestion that Teensy firmware would require C++.

The desktop executable does not run unchanged on a microcontroller. HID/MIDI
transports, filesystem/process facilities, and dependency suitability need
embedded adaptation. The BSP's device driver does not establish a complete
host-side HID and multichannel audio implementation.

[TinyUSB now documents UAC1/UAC2 host audio][tinyusb-audio], including explicit
feedback. Its README's host-class inventory omitted Audio when checked, so the
separate implementation matters. The example defaults to mono/stereo S16_LE;
its configurable stream API is not proof of S4 compatibility. It does not use
implicit-feedback audio endpoints to pace playback and has limits on UAC2 clock
arrangements. Rust integration with a C stack is possible, but no complete Rust
S4 audio bridge was established.

The current [input path](../src/input.rs) requests fresh HID input reports 1 and
3, processes touch/buttons before jog movement, and rejects stale interrupt
copies. Hardware translation must preserve those semantics and held-control
ownership; an interrupt-only example is not equivalent to the current bridge.

## Candidate boards

Retail prices were checked during October 7-8 unless marked historical or
unverified. Availability and prices can change. These are candidates, not boards
proven to satisfy the complete audio requirement.

| Board | Price evidence | RAM / storage | USB fit and remaining work |
| --- | --- | --- | --- |
| [Luckfox Lyra][lyra] | [USD 18.99][lyra-shop], listed in stock | 128 MB / microSD required | Two USB 2.0 OTG ports; host connection needs an MX1.25 adapter. Vendor gadget image remains unverified. |
| [Luckfox Lyra B][lyra] | [USD 25.99][lyra-b-shop], listed in stock | 128 MB / 256 MB NAND | Same USB arrangement; NAND avoids a separate boot card, not the audio software work. |
| [Luckfox Lyra Ultra, non-W][lyra-ultra] | [USD 36.99][lyra-ultra-shop], listed in stock | 512 MB / 8 GB eMMC | Two USB 2.0 OTG paths and onboard USB-A; useful storage/connector upgrade. Exact gadget kernel still needs verification. |
| Orange Pi Zero 2W | Historical offer around USD 16; current checkout price unverified | Configuration-dependent RAM / microSD | [Mainline DTS][zero2w-dts] explicitly enables peripheral USB plus a separate EHCI host. Check image and accessories. |
| [Radxa ZERO 3W][zero3-docs] | [USD 39.99][zero3-shop], available 2 GB/no-eMMC/header variant | 2 GB / microSD required for that variant | Separate USB 3 host and USB 2 OTG ports. Needs a USB-C host adapter and image verification. |
| [Sipeed Longan Pi 3H][longan] | [Historical launch pricing roughly USD 24-35][longan-price]; current price unverified | Configuration-dependent RAM/eMMC | Two USB-A host ports plus USB-C OTG. Convenient topology; shipped gadget support unverified. |
| [Raspberry Pi 4, 1 GB][pi4-shop] | USD 35, listed in stock | 1 GB / microSD required | Separate USB-A hosts and [USB-C gadget support][pi-gadget]. Established Linux prototype platform; power arrangement needs validation. |
| [Raspberry Pi 5, 1 GB][pi5-shop] | USD 45 listed; local offer below | 1 GB / microSD required | Separate USB-A hosts and documented USB-C gadget support. More CPU headroom, with power/cooling and assembly costs to assess. |
| [Teensy 4.0][teensy40] | [USD 23.80][teensy40-shop]; local offer below | 1 MB / onboard firmware flash | Independent 480 Mbit/s host/device. Host data is on underside pads; external host power circuitry is needed. |
| [Teensy 4.1][teensy41] | [USD 31.50][teensy41-shop]; local offer below | 1 MB / onboard firmware flash | Independent 480 Mbit/s host/device, host header and power management. Embedded multichannel audio integration remains substantial. |

[RK3506 documentation][rk3506] supports the Lyra family's two high-speed-capable
OTG controllers. [Luckfox's kernel guide][lyra-kernel] demonstrates enabling
`SND_USB_AUDIO` in its Linux 6.1 SDK; this is host support, not proof that the
shipping image enables the UAC2/MIDI gadget functions.

Raspberry Pi's documented gadget setup is Ethernet, not a ready audio/MIDI
bridge. It explicitly notes that computer USB power may be insufficient.
The Pi 5's additional CPU headroom may help resampling or clock correction;
neither a lower latency result nor mandatory active cooling for this workload
was established.

### Deprioritized and exploratory alternatives

| Family / board | Reason or unresolved issue |
| --- | --- |
| RP2040/RP2350 with PIO USB | Native and PIO links are Full-Speed, not a suitable default for unrestricted multichannel 24-bit audio. |
| ESP32-P4 | One High-Speed and one Full-Speed OTG controller, not two High-Speed links. |
| ESP32-S2/S3 and typical small Arduino boards | No established pair of independent High-Speed host/device links for this workload. |
| Cheap STM32F4/F7/H743 boards | The HS controller may need an external ULPI PHY to reach 480 Mbit/s; the other controller is commonly FS. Exact part and schematic matter. |
| CH32V307 | HS/FS split, plus [TinyUSB's explicit USB-audio warning][tinyusb-issues] about overwritten isochronous completion status. |
| Raspberry Pi Zero 2 W | One data controller cannot independently host the S4 while presenting the gadget. |
| Milk-V Duo S / Luckfox Pico Ultra | The researched connector arrangements switch roles rather than provide the required independent simultaneous paths. |
| Orange Pi Zero 3 | No verified gadget data path through the researched USB-C power connector; do not confuse it with Zero 2W. |
| ROCK Pi S / S0 | Suitable host/OTG topology, but [an RK3308 UAC2 report][rk3308-report] describes large-packet stalls. It is a concern to investigate, not proof of failure at the S4 workload or of a current fix. |
| [CH32H417][ch32h417] | Newer SS/HS/FS controller combination worth watching; complete audio-stack, simultaneous-role, and Rust support were not established. Do not generalize the CH32V307 warning to this different chip. |

## Brooklyn Micro Center availability snapshot

Checked October 8, 2026 in the user's Chrome browser with Brooklyn selected
(store ID 115). All four pages of the [94 in-stock development-board results][mc-search]
were read. This is a dated inventory observation, not a stock guarantee.

| Relevant board | Price before tax | Displayed stock | SKU |
| --- | ---: | ---: | --- |
| [Raspberry Pi 5, 1 GB][mc-pi5] | USD 44.99 | 9 new | 959668 |
| [Teensy 4.1][mc-teensy41] | USD 31.99 | 1 | 911750 |
| [Teensy 4.0][mc-teensy40] | USD 23.99 | 1 | 911768 |

The Pi 5 product page confirmed Brooklyn stock and aisle 16. It suggested a
USD 13.99 Micro Center 32 GB microSD card (SKU 482083), giving USD 58.98 for board
plus that card before tax, excluding power and other additions.

A Pi 400 was also listed at USD 44.99 with 25+ in stock; its keyboard form factor
was not preferred and its USB gadget path was not assessed in this inventory
check. A standalone Pi 4 1 GB, Lyra, and Radxa were not among these in-stock
development-board results. This does not establish their absence from every
Micro Center category or other store.

## Recommendation and unresolved checks

At the original budget, Lyra and Zero 2W were the Linux leads, with Teensy 4.0
the cheaper MCU comparison. At USD 50, Pi 4/Pi 5 are leading Linux prototype
candidates; Lyra Ultra and Radxa remain compact alternatives. Given the observed
Brooklyn inventory, Pi 5 1 GB was the preferred locally available prototype
option. This is a recommendation, not a user hardware-selection decision.

Before purchase, resolve the board-versus-assembly budget, complete S4 audio
descriptors, and the chosen image/stack's host and composite gadget support.
Numeric added-latency limits, monitored-input round-trip limits, tolerated
dropouts, and soak duration also need agreement. Software simulation can help
with descriptor checks, but cannot prove physical-link performance.

After obtaining hardware, the decisive verification is:

- Simultaneous forwarding of every required capture and playback channel at
  the selected supported formats, without silently reducing resolution.
- Mac/Djay enumeration of composite audio and MIDI, with correct input,
  master/headphone, and hardware-local volume/source behavior.
- Measured playback and capture delay versus direct S4 connection, and
  software-monitored input round-trip delay where applicable.
- Stable buffer occupancy and no underruns/overruns during an agreed soak,
  including sustained HID snapshots, jog input, and returning LED/meter traffic.
- Reliable boot, power, disconnect/reconnect, and sample-rate transitions.

No candidate has passed these checks. No performance claim follows from a
datasheet, gadget example, RAM calculation, or local stock listing.

## Sources

External links describe source evidence, not project hardware test results.
Retail and inventory observations are dated above; live listings may change.

[ni-inputs]: https://support.native-instruments.com/support/solutions/articles/69000879459-how-to-use-a-microphone-with-the-traktor-kontrol-s4
[mixxx]: https://manual.mixxx.org/2.7/en/hardware/controllers/native_instruments_traktor_kontrol_s4_mk3
[usb-device]: https://docs.rs/usb-device/latest/usb_device/
[linux-gadgets]: https://docs.kernel.org/usb/gadget-testing.html
[linux-audio]: https://github.com/torvalds/linux/blob/master/drivers/usb/gadget/function/u_audio.c
[alsaloop]: https://github.com/alsa-project/alsa-utils/blob/master/alsaloop/alsaloop.1
[teensy-rust]: https://docs.rs/teensy4-bsp/latest/teensy4_bsp/
[imxrt-usbd]: https://docs.rs/imxrt-usbd/0.4.2/imxrt_usbd/
[tinyusb-audio]: https://docs.tinyusb.org/en/latest/examples/host/audio_host.html
[tinyusb-issues]: https://docs.tinyusb.org/en/latest/reference/device_issues.html
[lyra]: https://www.luckfox.com/Luckfox-Lyra
[lyra-shop]: https://www.waveshare.com/luckfox-lyra.htm?sku=29762
[lyra-b-shop]: https://www.waveshare.com/luckfox-lyra.htm?sku=29751
[lyra-ultra]: https://www.luckfox.com/Luckfox-Lyra-Ultra
[lyra-ultra-shop]: https://www.waveshare.com/luckfox-lyra-ultra.htm?sku=30511
[lyra-kernel]: https://wiki.luckfox.com/Luckfox-Lyra/Kernel-Configuration/
[rk3506]: https://opensource.rock-chips.com/images/5/51/Rockchip_RK3506G2_Datasheet_V1.3-20250811.pdf
[zero2w-dts]: https://github.com/torvalds/linux/blob/master/arch/arm64/boot/dts/allwinner/sun50i-h618-orangepi-zero2w.dts
[zero3-docs]: https://docs.radxa.com/en/zero/zero3/hardware-design/hardware-interface
[zero3-shop]: https://arace.tech/products/radxa-zero-3w
[longan]: https://wiki.sipeed.com/hardware/en/longan/h618/lpi3h/1_intro.html
[longan-price]: https://liliputing.com/sipeed-longan-pi3h-is-a-raspberry-pi-zero-sized-board-with-full-sized-ethernet-usb-and-hdmi-ports/
[pi4-shop]: https://www.pishop.us/product/raspberry-pi-4-model-b-1gb/
[pi5-shop]: https://www.pishop.us/product/raspberry-pi-5-1gb/
[pi-gadget]: https://www.raspberrypi.com/news/usb-gadget-mode-in-raspberry-pi-os-ssh-over-usb/
[teensy40]: https://www.pjrc.com/store/teensy40.html
[teensy41]: https://www.pjrc.com/store/teensy41.html
[teensy40-shop]: https://www.sparkfun.com/teensy-4-0.html
[teensy41-shop]: https://www.sparkfun.com/teensy-4-1.html
[rk3308-report]: https://www.spinics.net/lists/linux-usb/msg252266.html
[ch32h417]: https://github.com/openwch/ch32h417
[mc-search]: https://www.microcenter.com/search/search_results.aspx?fq=Component+Type%3ADevelopment+Board&storeID=115
[mc-pi5]: https://www.microcenter.com/product/704295/raspberry-pi-5-1gb?storeID=115
[mc-teensy41]: https://www.microcenter.com/product/699941/pjrccom-teensy-41-development-board?storeID=115
[mc-teensy40]: https://www.microcenter.com/product/699942/pjrccom-teensy-40-development-board?storeID=115
