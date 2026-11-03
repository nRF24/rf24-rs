pub const TX_NORMAL: u8 = 0;
pub const TX_ROUTED: u8 = 1;
pub const NUM_PIPES: u8 = if cfg!(feature = "8-pipes") {
    8
} else if cfg!(feature = "7-pipes") {
    7
} else {
    6
};

/// A reserved valid address for use with RF24Mesh (when a mesh node requests an assigned address)
pub(crate) const NETWORK_DEFAULT_ADDRESS: u16 = 0o4444;

/// A sentinel address value for multicasting purposes
pub(crate) const NETWORK_MULTICAST_ADDRESS: u16 = 0o100;
pub(crate) const NETWORK_MULTICAST_ADDRESS_LEVEL_2: u16 = 0o10;
pub(crate) const NETWORK_MULTICAST_ADDRESS_LEVEL_4: u16 = 0o1000;

/// A sentinel value for internally indicating that the frame should be automatically routed as necessary
pub(crate) const NETWORK_AUTO_ROUTING: u8 = 0o70;

#[derive(Debug, Default, Clone, Copy)]
pub struct LogicalRoute {
    /// The immediate destination (1 hop) of an outgoing frame
    pub send_node: u16,
    /// The pipe number of the `send_node` for which outgoing packets are aimed at
    pub send_type: u8,
    /// A flag to indicate that the outgoing frame does not want an auto-ack from `send_node`
    pub multicast: bool,
}

impl LogicalRoute {
    pub fn prepare(self, node_address: &LogicalAddress) -> Self {
        //Create pointers so this makes sense.. kind of
        //We take in the to_node(logical) now, at the end of the function, output the send_node(physical) address, etc.
        //back to the original memory address that held the logical information.

        let mut converted = self;

        // Where do we send this?  By default, to our parent.
        let mut pre_conversion_send_node = node_address.parent_node;

        // On which pipe? By default, to out parent's listening pipe.
        let mut pre_conversion_send_pipe = node_address.parent_pipe;

        if self.send_type > TX_ROUTED {
            pre_conversion_send_node = self.send_node;
            converted.multicast = true;
            pre_conversion_send_pipe = 0;
        } else if node_address.is_descendant(&self.send_node) {
            pre_conversion_send_pipe = NUM_PIPES - 1; // Send to its listening pipe
                                                      //
            if node_address.is_direct_child(&self.send_node) {
                // If the node is a direct child, then send it directly
                pre_conversion_send_node = self.send_node;
            }
            // If the node is a child of a child
            // talk on our child's listening pipe,
            // and let the direct child relay it.
            else {
                pre_conversion_send_node = node_address.direct_child_route_to(&self.send_node);
            }
        }

        converted.send_node = pre_conversion_send_node;
        converted.send_type = pre_conversion_send_pipe;
        converted
    }
}

pub struct LogicalAddress {
    pub address: u16,
    pub(crate) parent_node: u16,
    pub(crate) parent_pipe: u8,
    pub(crate) node_mask: u16,
    #[cfg(feature = "multicast")]
    pub(crate) multicast_level: u8,
}

impl LogicalAddress {
    pub fn new(node_address: u16) -> Self {
        // First, establish the node_mask
        let mut node_mask_check = u16::MAX;
        #[cfg(feature = "multicast")]
        let mut count = 0u8;

        while (node_address & node_mask_check) > 0 {
            node_mask_check <<= 3;
            #[cfg(feature = "multicast")]
            {
                count += 1;
            }
        }
        #[cfg(feature = "multicast")]
        let multicast_level = count;

        let node_mask = !node_mask_check;

        // parent mask is the next level down
        let parent_mask = node_mask >> 3;

        // parent node is the part IN the mask
        let parent_node = node_address & parent_mask;

        // parent pipe is the part OUT of the mask
        let mut i = node_address;
        let mut m = parent_mask;
        while m > 0 {
            i >>= 3;
            m >>= 3;
        }
        let parent_pipe = (i & 0xFF) as u8;

        Self {
            address: node_address,
            parent_node,
            parent_pipe,
            node_mask,
            #[cfg(feature = "multicast")]
            multicast_level,
        }
    }

    /// A direct child of ours has the same low numbers as us, and only
    /// one higher number.
    ///
    /// e.g. node 0234 is a direct child of 034, and node 01234 is a
    /// descendant but not a direct child
    pub(crate) fn is_direct_child(&self, to_node: &u16) -> bool {
        // Does it only have ONE more level than us?
        let child_node_mask = !self.node_mask << 3;
        (*to_node & child_node_mask) == 0
    }

    pub(crate) fn is_descendant(&self, to_node: &u16) -> bool {
        (*to_node & self.node_mask) == self.address
    }

    pub(crate) fn direct_child_route_to(&self, to_node: &u16) -> u16 {
        // Presumes that this is in fact a child!!
        let child_mask = (self.node_mask << 3) | 0x07;
        *to_node & child_mask
    }
}

const ADDRESS_PREFIX: u8 = 0xCC;
const ADDRESS_SUFFIX: [u8; 6] = [
    0xC3,
    0x3C,
    0x33,
    0xCE,
    0x3E,
    0xE3,
    #[cfg(feature = "7-pipes")]
    0xEE,
    #[cfg(feature = "8-pipes")]
    0xED,
];
/// Translate logical `node` address into physical address for the given `pipe` number.
pub(crate) fn pipe_address(node: u16, pipe: u8) -> [u8; 5] {
    let pipe = pipe.min(NUM_PIPES - 1);
    let mut result = [ADDRESS_PREFIX; 5];
    let mut count = 1;
    let mut dec = node;
    while dec > 0 {
        #[cfg(feature = "multicast")]
        {
            if pipe > 0 || node == 0 {
                result[count] = ADDRESS_SUFFIX[dec as usize % 8];
            }
        }
        #[cfg(not(feature = "multicast"))]
        {
            result[count] = ADDRESS_SUFFIX[dec as usize % 8];
        }
        dec >>= 3;
        count += 1;
    }

    #[cfg(feature = "multicast")]
    {
        if pipe > 0 || node == 0 {
            result[0] = ADDRESS_SUFFIX[pipe as usize];
        } else {
            result[1] = ADDRESS_SUFFIX[count - 1];
        }
    }
    #[cfg(not(feature = "multicast"))]
    {
        result[0] = ADDRESS_SUFFIX[pipe as usize];
    }

    #[cfg(test)]
    {
        extern crate std;
        std::println!("{node:#04o} for pipe {pipe} is {result:02X?}");
    }
    result
}

/// Translate a network level number (0-3) to a Logical address (used for TX multicasting)
#[cfg(feature = "multicast")]
pub(crate) fn level_to_address(level: u8) -> u16
{
    if level > 0 {
        1 << ((level - 1) * 3)
    } else {
        0
    }
}

/// Validate a network node address as a proper logical address.
///
/// Addresses are specified in octal form, ie `0o11`, `0o34`.
/// Review [RF24Network addressing](addressing.md) for more information.
/// This function will validate an improper address of `0o10`, `0o100`, or `0o1000` because
/// they are reserved for multicasting.
pub fn is_valid_address(node: u16) -> bool
{
    let mut result = true;
    if matches!(node, NETWORK_MULTICAST_ADDRESS | NETWORK_MULTICAST_ADDRESS_LEVEL_2 | NETWORK_MULTICAST_ADDRESS_LEVEL_4) {
        return result;
    }
    let mut count = 0u8;
    let mut node = node;
    while node > 0 {
        let digit = (node & 0x07) as u8;
        if digit < 1 || digit > (NUM_PIPES - 1) {
            result = false;
            break;
        }
        node >>= 3;
        count += 1;
    }

    if count > 4 {
        return false;
    }
    result
}
