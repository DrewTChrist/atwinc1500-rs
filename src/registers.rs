pub const WIFI_HOST_RCV_CTRL_0: u32 = 0x1070;
pub const WIFI_HOST_RCV_CTRL_1: u32 = 0x1084;
pub const WIFI_HOST_RCV_CTRL_2: u32 = 0x1078;
pub const WIFI_HOST_RCV_CTRL_3: u32 = 0x106c;
pub const WIFI_HOST_RCV_CTRL_4: u32 = 0x150400;
pub const WIFI_HOST_RCV_CTRL_5: u32 = 0x1088;
pub const NMI_CHIPID: u32 = 0x1000;
// The efuse register is a magic number
// in the atmel driver and is not
// explicitly listed in the datasheet
pub const EFUSE_REG: u32 = 0x1014;
pub const NMI_STATE_REG: u32 = 0x108c;
pub const NMI_PIN_MUX_0: u32 = 0x1408;
#[allow(non_upper_case_globals)]
pub const rNMI_GP_REG_1: u32 = 0x14a0;
#[allow(non_upper_case_globals)]
pub const rNMI_GP_REG_2: u32 = 0xc0008;
pub const NMI_INTR_REG_BASE: u32 = 0x1a00;
pub const NMI_SPI_PROTOCOL_CONFIG: u32 = 0xe824;
pub const BOOTROM_REG: u32 = 0xc000c;
pub const M2M_WAIT_FOR_HOST_REG: u32 = 0x207bc;
pub const CORT_HOST_COMM: u32 = 0x10;
pub const HOST_CORT_COMM: u32 = 0x0b;
pub const WAKE_CLK_REG: u32 = 0x1;
pub const CLOCKS_EN_REG: u32 = 0xf;
pub const NMI_PERIPH_REG_BASE: u32 = 0x1000;
#[allow(non_upper_case_globals)]
pub const rNMI_GP_REG_0: u32 = 0x149c;
#[allow(non_upper_case_globals)]
pub const rNMI_GLB_RESET: u32 = 0x1400;
#[allow(non_upper_case_globals)]
pub const rNMI_BOOT_RESET_MUX: u32 = 0x1118;
pub const NMI_REV_REG: u32 = 0x207ac;
pub const NMI_REV_REG_ATE: u32 = 0x1048;
pub const M2M_FINISH_INIT_STATE: u32 = 0x02532636;
pub const M2M_FINISH_BOOT_ROM: u32 = 0x10add09e;
pub const M2M_START_FIRMWARE: u32 = 0xef522f61;
pub const M2M_START_PS_FIRMWARE: u32 = 0x94992610;
pub const M2M_ATE_FW_START_VALUE: u32 = 0x3C1CD57D;
pub const M2M_ATE_FW_IS_UP_VALUE: u32 = 0xD75DC1C3;
