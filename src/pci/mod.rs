use core::{fmt, ptr::addr_of_mut};

use bit_field::BitField;

use crate::arch::{PAddr, VAddr, PciInterface};

pub mod device_db;

pub type VendorId = u16;
pub type DeviceId = u16;
pub type DeviceRevision = u8;
pub type BaseClass = u8;
pub type SubClass = u8;
pub type Interface = u8;
pub type HeaderType = u8;

#[derive(Debug)]
pub enum PciDeviceType {
    Endpoint = 0x00,
    PciBridge = 0x01,
    Unknown = 0xff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PCIAddress {
    pub bus: u8,
    pub dev: u8,
    pub fun: u8,
}

impl PCIAddress {
    fn new(bus: u8, dev: u8, fun: u8) -> Self {
        assert!(dev <= 31);
        assert!(fun <= 7);

        //trace!("address ({:2}:{:2}.{:1})", bus, dev, fun);
        PCIAddress { bus, dev, fun }
    }

    pub fn addr(&self) -> u32 {
        (1 << 31) | ((self.bus as u32) << 16) | ((self.dev as u32) << 11) | ((self.fun as u32) << 8)
    }
}

impl fmt::Debug for PCIAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}.{}", self.bus, self.dev, self.fun)
    }
}

#[derive(Debug)]
pub struct PCIHeader(PCIAddress);

impl PCIHeader {
    pub fn new(bus: u8, device: u8, function: u8) -> Option<Self> {
        let addr = PCIAddress::new(bus, device, function);
        if PCIHeader::is_valid(addr) {
            Some(PCIHeader(addr))
        } else {
            None
        }
    }

    pub fn is_valid(addr: PCIAddress) -> bool {
        addr.read(0) != u32::MAX
    }
}

/// # See also
/// <https://wiki.osdev.org/PCI#Class_Codes>
#[derive(Debug)]
pub enum ClassCode {
    IDEController = 0x0101,
    SATAController = 0x0106,
    EthernetController = 0x0200,
    VGACompatibleController = 0x0300,
    RAMController = 0x0500,
    HostBridge = 0x0600,
    ISABridge = 0x0601,
    OtherBridge = 0x0680,
    Unknown = 0xffff,
}

impl From<u16> for ClassCode {
    fn from(value: u16) -> ClassCode {
        match value {
            0x0101 => ClassCode::IDEController,
            0x0106 => ClassCode::SATAController,
            0x0200 => ClassCode::EthernetController,
            0x0300 => ClassCode::VGACompatibleController,
            0x0500 => ClassCode::RAMController,
            0x0600 => ClassCode::HostBridge,
            0x0601 => ClassCode::ISABridge,
            0x0680 => ClassCode::OtherBridge,
            _ => ClassCode::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BarType {
    IO,
    Mem,
}

impl From<bool> for BarType {
    fn from(value: bool) -> BarType {
        match value {
            true => BarType::IO,
            false => BarType::Mem,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub region_type: BarType,
    pub prefetchable: bool,
    pub address: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityId {
    /// Null Capability
    ///
    /// This capability contains no registers other than those described below.
    /// It may be present in any Function. Functions may contain multiple
    /// instances of this capability. The Null Capability is 16 bits and
    /// contains an 8-bit Capability ID followed by an 8-bit Next Capability
    /// Pointer.
    Null,
    /// PCI Power Management Interface
    ///
    /// This Capability structure provides a standard interface to control power
    /// management features in a device Function. It is fully documented in the
    /// PCI Bus Power Management Interface Specification.
    PowerManagement,
    /// AGP
    ///
    /// This Capability structure identifies a controller that is capable of
    /// using Accelerated Graphics Port features. Full documentation can be
    /// found in the Accelerated Graphics Port Interface Specification.
    Agp,
    /// VPD
    ///
    /// This Capability structure identifies a device Function that supports
    /// Vital Product Data. Full documentation of this feature can be found in
    /// the PCI Local Bus Specification.
    Vdp,
    /// Slot Identification
    ///
    /// This Capability structure identifies a bridge that provides external
    /// expansion capabilities. Full documentation of this feature can be found
    /// in the PCI-to-PCI Bridge Architecture Specification.
    SlotIdent,
    /// Message Signaled Interrupts
    ///
    /// This Capability structure identifies a device Function that can do
    /// message signaled interrupt delivery. Full documentation of this feature
    /// can be found in the PCI Local Bus Specification.
    Msi,
    /// CompactPCI Hot Swap
    ///
    /// This Capability structure provides a standard interface to control and
    /// sense status within a device that supports Hot Swap insertion and
    /// extraction in a CompactPCI system. This Capability is documented in the
    /// CompactPCI Hot Swap Specification PICMG 2.1, R1.0 available at
    /// http://www.picmg.org.
    CompactPci,
    /// PCI-X
    ///
    /// Refer to the PCI-X Protocol Addendum to the PCI Local Bus Specification
    /// for details.
    PciX,
    /// HyperTransport
    ///
    /// This Capability structure provides control and status for devices that
    /// implement HyperTransport Technology links. For details, refer to the
    /// HyperTransport I/O Link Specification available at
    /// http://www.hypertransport.org.
    HyperTransport,
    /// Vendor Specific
    ///
    /// This Capability structure allows device vendors to use the Capability
    /// mechanism to expose vendor-specific registers. The byte immediately
    /// following the Next Pointer in the Capability structure is defined to be
    /// a length field. This length field provides the number of bytes in the
    /// Capability structure (including the Capability ID and Next Pointer
    /// bytes). All remaining bytes in the capability structure are
    /// vendor-specific.
    VendorSpecific,
    /// Debug port
    DebugPort,
    /// CompactPCI central resource control
    ///
    /// Definition of this Capability can be found in the PICMG 2.13
    /// Specification (http://www.picmg.com).
    CompactPCI,
    /// PCI Hot-Plug
    ///
    /// This Capability ID indicates that the associated device conforms to the
    /// Standard Hot-Plug Controller model.
    HotPlug,
    /// PCI Bridge Subsystem Vendor ID
    BridgeSubsystemVendor,
    /// AGP 8x
    Agp8,
    /// Secure Device
    SecureDevice,
    /// PCI Express
    PCIExpress,
    /// MSI-X
    ///
    /// This Capability ID identifies an optional extension to the basic MSI
    /// functionality.
    MsiX,
    /// Serial ATA Data/Index Configuration
    SerialAtaConfiguration,
    /// Advanced Features (AF)
    ///
    /// Full documentation of this feature can be found in the Advanced
    /// Capabilities for Conventional PCI ECN.
    AdvancedFeatures,
    /// Enhanced Allocation
    EnhancedAllocation,
    /// Flattening Portal Bridge
    FlatteningPortalBridge,
    /// Reserved
    Unknown(u8),
}

impl From<u8> for CapabilityId {
    fn from(capid: u8) -> Self {
        match capid {
            0x00 => CapabilityId::Null,
            0x01 => CapabilityId::PowerManagement,
            0x02 => CapabilityId::Agp,
            0x03 => CapabilityId::Vdp,
            0x04 => CapabilityId::SlotIdent,
            0x05 => CapabilityId::Msi,
            0x06 => CapabilityId::CompactPci,
            0x07 => CapabilityId::PciX,
            0x08 => CapabilityId::HyperTransport,
            0x09 => CapabilityId::VendorSpecific,
            0x0A => CapabilityId::DebugPort,
            0x0B => CapabilityId::CompactPCI,
            0x0C => CapabilityId::HotPlug,
            0x0D => CapabilityId::BridgeSubsystemVendor,
            0x0E => CapabilityId::Agp8,
            0x0F => CapabilityId::SecureDevice,
            0x10 => CapabilityId::PCIExpress,
            0x11 => CapabilityId::MsiX,
            0x12 => CapabilityId::SerialAtaConfiguration,
            0x13 => CapabilityId::AdvancedFeatures,
            0x14 => CapabilityId::EnhancedAllocation,
            0x15 => CapabilityId::FlatteningPortalBridge,
            x => CapabilityId::Unknown(x),
        }
    }
}

pub enum CapabilityType<'s> {
    MsiX(MsiX<'s>),
    Unknown(CapabilityId),
}

#[derive(Debug)]
pub struct MsiX<'s> {
    /// A reference to the device's PCI header.
    header: &'s mut PCIHeader,
    /// The offset where the MSI-X config is located within the PCI header.
    pub offset: u32,
}

impl<'s> MsiX<'s> {

    pub fn message_control(&self) -> u16 {
        (self.header.0.read(self.offset) >> 16) as u16
    }

    pub fn enabled(&self) -> bool {
        self.message_control().get_bit(15)
    }

    pub fn enable(&mut self) {
        let ctrl = *self.message_control().set_bit(15, true);

        let mut hdr = self.header.0.read(self.offset);
        hdr = (hdr & 0xFFFF) | ((ctrl as u32) << 16);
        self.header.0.write(self.offset, hdr);
    }

    pub fn function_mask(&self) -> bool {
        self.message_control().get_bit(14)
    }

    /// Table Size is N - 1 encoded, and is the number of entries in the MSI-X
    /// table.
    ///
    /// This field is Read-Only.
    pub fn table_size(&self) -> usize {
        self.message_control().get_bits(0..10) as usize
    }

    /// BIR specifies which BAR is used for the Message Table.
    ///
    /// This may be a 64-bit BAR, and is zero-indexed (so BIR=0, BAR0, offset
    /// 0x10 into the header).
    pub fn bir(&self) -> u8 {
        (self.header.0.read(self.offset + 4) & 0b111) as u8
    }

    /// Table Offset is an offset into that BAR where the Message Table lives.
    ///
    /// Note that it is 8-byte aligned.
    pub fn table_offset(&self) -> u32 {
        self.header.0.read(self.offset + 4) & !0b111
    }


    /// BIR specifies which BAR is used for the Message Table.
    ///
    /// This may be a 64-bit BAR, and is zero-indexed (so BIR=0, BAR0, offset
    /// 0x10 into the header).
    pub fn pending_bit_bir(&self) -> u8 {
        (self.header.0.read(self.offset + 8) & 0b111) as u8
    }

    /// Table Offset is an offset into that BAR where the Message Table lives.
    ///
    /// Note that it is 8-byte aligned.
    pub fn pending_bit_table_offset(&self) -> u32 {
        self.header.0.read(self.offset + 8) & !0b111
    }
}


#[derive(Debug)]
#[repr(C)]
pub struct MsiXTableEntry {
    addr: u64,
    data: u32,
    vector_control: u32,
}


#[derive(Debug)]
pub struct Capability {
    /// The (parsed) ID of the capability (read from bits 0..8 at offset).
    pub id: CapabilityId,
    /// The offset where the capability is located within the PCI header.
    pub offset: u8,
}

pub struct CapabilitiesIter<'s> {
    header: &'s PCIHeader,
    next: u8,
}

impl<'s> Iterator for CapabilitiesIter<'s> {
    type Item = Capability;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == 0 {
            return None;
        }

        let cap_header = self.header.0.read(self.next as u32);
        let id = CapabilityId::from(cap_header.get_bits(0..8) as u8);
        let cap = Capability {
            id,
            offset: self.next,
        };

        self.next = cap_header.get_bits(8..16) as u8;
        Some(cap)
    }
}

#[derive(Debug)]
pub struct PciDevice {
    header: PCIHeader,
}

impl PciDevice {
    pub fn new(bus: u8, device: u8, function: u8) -> Option<Self> {
        let header = PCIHeader::new(bus, device, function);
        header.map(|header| PciDevice { header })
    }

    pub fn pci_address(&self) -> PCIAddress {
        self.header.0
    }

    pub fn device_type(&self) -> PciDeviceType {
        let header = self.header.0.read(0x0c);

        match header.get_bits(16..23) as u8 {
            0x00 => PciDeviceType::Endpoint,
            0x01 => PciDeviceType::PciBridge,
            _ => PciDeviceType::Unknown,
        }
    }

    pub fn get_cap_region_mut(&mut self, cap: Capability) -> CapabilityType {
        match cap.id {
            CapabilityId::MsiX => CapabilityType::MsiX(MsiX { header: &mut self.header, offset: cap.offset as u32 }),
            _ => unimplemented!(),
        }
    }

    fn get_msix_config(&mut self) -> Option<MsiX> {
        self.capabilities().find(|cap| cap.id == CapabilityId::MsiX).map(move |cap| {
            MsiX { header: &mut self.header, offset: cap.offset as u32 }
        })
    }

    pub fn get_msix_irq_table_mut(&mut self, paddr_to_vaddr_conversion: &Fn(PAddr) -> VAddr) -> Option<&mut [MsiXTableEntry]> {

        if let Some(mut msi) = self.get_msix_config() {
            log::info!("Device has MSI-X capability and it's {}", if msi.enabled() { "enabled" } else { "not enabled" });
            if !msi.enabled() {
                msi.enable();
            }
            log::info!("Device has MSI-X capability and it's {}", if msi.enabled() { "enabled" } else { "not enabled" });
            log::info!("Device MSI-X table is at bar {} offset {} table size is {}", msi.bir(), msi.table_offset(), msi.table_size());

            let table_bar = msi.bir();
            let table_offset = msi.table_offset();

            let entries = msi.table_size() + 1;
            let bar = self.bar(table_bar).unwrap();
            let addr = paddr_to_vaddr_conversion(PAddr::from(bar.address + table_offset as u64));

            // Safety:
            // - We're casting the part of the memory to a MSI-X table according to the spec
            // - It's just plain-old-data
            // - We have &mut self when giving out a mut reference to the table
            // - Sanity check that we're within `bar`'s range (TODO)
            // - Check that `addr` satisfies alignment for [MsiXTableEntry] (TODO)
            let msix_table = unsafe { core::slice::from_raw_parts_mut(addr.as_mut_ptr::<MsiXTableEntry>(), entries) };
            return Some(msix_table);
        } else {
            return None;
        }
    }

    pub fn vendor_id(&self) -> VendorId {
        self.header.0.read(0x00) as VendorId
    }

    pub fn device_id(&self) -> DeviceId {
        self.header.0.read(0x02) as DeviceId
    }

    pub fn is_bus_master(&self) -> bool {
        self.header.0.read(0x04).get_bit(2)
    }

    pub fn enable_bus_mastering(&mut self) {
        let mut command = self.header.0.read(0x04);
        command.set_bit(2, true);
        self.header.0.write(0x04, command);
    }

    pub fn bar(&mut self, index: u8) -> Option<Bar> {
        match self.device_type() {
            PciDeviceType::Endpoint => assert!(index < 6),
            PciDeviceType::PciBridge => assert!(index < 2),
            PciDeviceType::Unknown => return None,
        }

        let offset = 0x10 + (index as u32) * 4;
        let base = self.header.0.read(offset);
        let bartype_is_io = base.get_bit(0);

        if !bartype_is_io {
            let locatable = base.get_bits(1..3);
            let prefetchable = base.get_bit(3);

            self.header.0.write(offset, u32::MAX);
            let size_encoded = self.header.0.read(offset);
            self.header.0.write(offset, base);

            if size_encoded == 0x0 {
                return None;
            }

            // To get the region size using BARs:
            // - Clear lower 4 bits
            // - Invert all all-bits
            // - Add 1 to the result
            // Ref: https://wiki.osdev.org/PCI#Base_Address_Registers
            let (address, size) = {
                match locatable {
                    // 32-bit address
                    0 => {
                        let size = !(size_encoded & !0xF) + 1;
                        ((base & 0xFFFF_FFF0) as u64, size as u64)
                    }
                    // 64-bit address
                    2 => {
                        let next_offset = offset + 4;
                        let next_bar = self.header.0.read(next_offset);
                        let address = (base & 0xFFFF_FFF0) as u64
                            | (next_bar as u64 & (u32::MAX as u64)) << 32;

                        // Size for 64-bit Memory Space BARs:
                        self.header.0.write(next_offset, u32::MAX);
                        let msb_size_encoded = self.header.0.read(next_offset);
                        self.header.0.write(next_offset, next_bar);
                        let size = (msb_size_encoded as u64) << 32 | size_encoded as u64;

                        (address, (!(size & !0xF) + 1))
                    }
                    _ => unimplemented!("Unsupported locatable: {}", locatable),
                }
            };

            Some(Bar {
                region_type: bartype_is_io.into(),
                prefetchable,
                address,
                size,
            })
        } else {
            unimplemented!("Unable to handle IO BARs")
        }
    }

    pub fn status(&self) -> u16 {
        (self.header.0.read(0x4) >> 16)as u16
    }

    /// Offset to capability pointer
    pub fn capabilities_pointer(&self) -> Option<u8> {
        let cap_ptr = self.header.0.read(0x34).get_bits(0..8) as u8;
        if self.status().get_bit(4) && cap_ptr != 0x0 {
            Some(cap_ptr)
        } else {
            None
        }
    }

    pub fn capabilities(&self) -> CapabilitiesIter {
        self.capabilities_pointer().map_or(CapabilitiesIter {
            header: &self.header,
            next: 0x0
        }, |cap_ptr| CapabilitiesIter {
            header: &self.header,
            next: cap_ptr
        })
    }

    pub fn revision_and_class(&self) -> (DeviceRevision, BaseClass, SubClass, Interface) {
        let field = { self.header.0.read(0x08) };
        (
            field.get_bits(0..8) as DeviceRevision,
            field.get_bits(24..32) as BaseClass,
            field.get_bits(16..24) as SubClass,
            field.get_bits(8..16) as Interface,
        )
    }

    pub fn device_class(&self) -> ClassCode {
        let (_revision, base_class, sub_class, _interface) = self.revision_and_class();
        let class = (base_class as u16) << 8 | (sub_class as u16);
        class.into()
    }

    pub fn info(&self) -> Option<&'static device_db::PciDeviceInfo> {
        let key = device_db::make_key(self.vendor_id(), self.device_id());
        crate::pci::device_db::PCI_DEVICES.get(&key)
    }
}

impl fmt::Display for PciDevice {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: ", self.header.0)?;
        if let Some(dev_info) = self.info() {
            write!(f, "{} {}", dev_info.vendor_name, dev_info.device_name)
        } else {
            write!(
                f,
                "Unknown[{:#x}] Unknown[{:#x}]",
                self.vendor_id(),
                self.device_id()
            )
        }
    }
}

pub struct PciDeviceIterator {
    bus: u8,
    device: u8,
    function: u8,
}

// Implement `Iterator` for `PciDeviceIterator`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl Iterator for PciDeviceIterator {
    type Item = PciDevice;

    fn next(&mut self) -> Option<Self::Item> {
        for bus in self.bus..=255 {
            for device in self.device..=31 {
                for function in self.function..=7 {
                    if let Some(pci_device) = PciDevice::new(bus, device, function) {
                        self.bus = bus;
                        self.device = device;
                        // Start with next function on next iteration
                        self.function = function + 1;

                        return Some(pci_device);
                    }
                }
                self.function = 0;
            }
            self.device = 0;
        }

        None
    }
}

/// Scans the PCI bus addresses, returns vector of all
pub fn scan_bus() -> PciDeviceIterator {
    PciDeviceIterator {
        bus: 0x0,
        device: 0x0,
        function: 0x0,
    }
}
