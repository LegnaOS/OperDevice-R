// 引入winapi库
extern crate winapi;
use std::os::windows::ffi::OsStringExt;
use std::env;
use std::ffi::OsString;

// 引入需要的Windows API函数和类型
use std::ptr::null_mut;
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::um::winnt::LPWSTR;
use winapi::um::winuser::{MessageBoxW, MB_OK};
use winapi::um::cfgmgr32::{CM_Get_Device_IDW, CM_Get_Device_ID_Size, CR_SUCCESS};
use winapi::um::setupapi::{SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiSetClassInstallParamsW, SetupDiChangeState, DIGCF_ALLCLASSES, DICS_FLAG_GLOBAL, DIF_PROPERTYCHANGE, SP_CLASSINSTALL_HEADER, SP_DEVINFO_DATA, SP_PROPCHANGE_PARAMS};

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    // 如果参数数量不为3，则打印使用方法并退出
    if args.len() != 3 {
        println!("使用方法: {} /enable|/disable device_name", args[0]);
        return;
    }

    // 获取设备名称，并将其转换为宽字符
    let device_name = OsString::from(&args[2]).encode_wide().collect::<Vec<u16>>();
    let device_name_ptr: LPWSTR = device_name.as_ptr() as LPWSTR;

    // 根据参数确定设备状态
    let state: DWORD = match args[1].as_str() {
        "/disable" => winapi::um::cfgmgr32::DICS_DISABLE,
        "/enable" => winapi::um::cfgmgr32::DICS_ENABLE,
        _ => return,
    };

    // 设置设备状态
    let result: BOOL = set_device_state(device_name_ptr, state);
    // 如果设置失败，则弹出消息框
    if result == 0 {
        let msg = OsString::from("改变设备状态失败").encode_wide().collect::<Vec<u16>>();
        unsafe {
            MessageBoxW(null_mut(), msg.as_ptr(), null_mut(), MB_OK);
        }
    }
}

// 设置设备状态的函数
fn set_device_state(device_name: LPWSTR, desired_state: DWORD) -> Result<(), &'static str> {
    let device_name_wide: Vec<u16> = OsString::from(device_name).encode_wide().collect();

    unsafe {
        // 获取设备信息集
        let dev_info = SetupDiGetClassDevsW(null_mut(), null_mut(), null_mut(), DIGCF_ALLCLASSES);
        if dev_info.is_null() {
            return Err("获取设备信息集失败");
        }

        let mut dev_data = SP_DEVINFO_DATA { cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32, ..Default::default() };
        let mut device_index = 0;

        // 遍历设备信息集
        while SetupDiEnumDeviceInfo(dev_info, device_index, &mut dev_data) != 0 {
            let mut id_size = 0;
            // 获取设备ID的大小
            CM_Get_Device_ID_Size(&mut id_size, dev_data.DevInst, 0);
            let mut id_buffer: Vec<u16> = vec![0; id_size as usize + 1];

            // 获取设备ID
            if CM_Get_Device_IDW(dev_data.DevInst, id_buffer.as_mut_ptr(), id_size + 1, 0) == CR_SUCCESS {
                // 如果设备ID与指定的设备名称相同，则设置设备状态
                if id_buffer == device_name_wide {
                    let mut prop_change_params = SP_PROPCHANGE_PARAMS {
                        ClassInstallHeader: SP_CLASSINSTALL_HEADER {
                            cbSize: std::mem::size_of::<SP_CLASSINSTALL_HEADER>() as u32,
                            InstallFunction: DIF_PROPERTYCHANGE,
                        },
                        StateChange: desired_state,
                        Scope: DICS_FLAG_GLOBAL,
                        HwProfile: 0,
                    };

                    // 设置设备安装参数
                    if SetupDiSetClassInstallParamsW(dev_info, &mut dev_data, &mut prop_change_params.ClassInstallHeader, std::mem::size_of::<SP_PROPCHANGE_PARAMS>() as u32) != 0 {
                        // 改变设备状态
                        if SetupDiChangeState(dev_info, &mut dev_data) != 0 {
                            // 销毁设备信息列表并返回成功
                            SetupDiDestroyDeviceInfoList(dev_info);
                            return Ok(());
                        } else {
                            return Err("改变设备状态失败");
                        }
                    } else {
                        return Err("设置设备安装参数失败");
                    }
                }
            }
            device_index += 1;
        }

        // 销毁设备信息列表并返回失败
        SetupDiDestroyDeviceInfoList(dev_info);
    }

    Err("设备未找到")
}