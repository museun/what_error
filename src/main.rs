use std::mem::MaybeUninit;

use winapi::{
    shared::minwindef::{HLOCAL, LPCVOID},
    shared::ntdef::{LANG_ENGLISH, MAKELANGID, SUBLANG_DEFAULT},
    shared::winerror::ERROR_MR_MID_NOT_FOUND,
    um::libloaderapi::{FreeLibrary, LoadLibraryExW, DONT_RESOLVE_DLL_REFERENCES},
    um::winbase::{
        FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_HMODULE,
        FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FORMAT_MESSAGE_MAX_WIDTH_MASK,
    },
    um::winnt::{LPWSTR, PWSTR},
};

#[derive(Debug)]
struct Args {
    error: String,
    nt_status: bool,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut args = pico_args::Arguments::from_env();

        if args.contains("-h") {
            print_short_help();
        }

        if args.contains("--help") {
            print_long_help();
        }

        if args.contains(["-v", "--version"]) {
            print_version();
        }

        let nt_status = args.contains("-nt");
        let error: String = args.value_from_str(["-e", "--error"])?;

        args.finish()?;

        Ok(Self { error, nt_status })
    }
}

const HEADER: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn print_long_help() -> ! {
    const HELP: &str = "
usage:
    -e, --error   <error_code>
        the eerror code to look up.
        this can be either as an integer or a hex literal
        e.g. -2147467261 or 0x80004003

    --nt
        assume its an NT status code.
        some error codes are shared (e.g. '5') between DOS and NT.
        this forces it to look it up as an NT status code

    -h, --help
        writes this help message

    -v, --version
        writes the current version
";

    println!("{}", HEADER);
    println!("{}", HELP);
    std::process::exit(0)
}

fn print_short_help() -> ! {
    const HELP: &str = "
usage:
    -e, --error    error code to lookup
    --nt           assume its an NT status code

    -h, --help     writes this help message
    -v, --version  writes the current version
    ";

    println!("{}", HEADER);
    println!("{}", HELP);
    std::process::exit(0)
}

fn print_version() -> ! {
    println!("{}", HEADER);
    std::process::exit(0)
}

fn format_error(code: i32) -> String {
    use std::os::windows::ffi::OsStrExt as _;

    unsafe fn pwstr_to_string(ptr: PWSTR) -> String {
        let len = (0..)
            .find(|&n| *ptr.add(n) == 0)
            .expect("expected null termination");

        let data = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(data)
    }

    let dll: Vec<u16> = std::ffi::OsStr::new("wininet.dll")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let module = unsafe {
        LoadLibraryExW(
            dll.as_ptr(),
            std::ptr::null_mut(),
            DONT_RESOLVE_DLL_REFERENCES,
        )
    };

    if module.is_null() {
        return "unknown".to_string();
    }

    let mut msg = MaybeUninit::<LPWSTR>::uninit();

    let ret = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_FROM_HMODULE
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS
                | FORMAT_MESSAGE_MAX_WIDTH_MASK
                | FORMAT_MESSAGE_ALLOCATE_BUFFER,
            module as LPCVOID,
            code as u32,
            MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT) as u32,
            msg.as_mut_ptr() as LPWSTR,
            // (&mut msg as *mut LPWSTR) as LPWSTR,
            0,
            std::ptr::null_mut(),
        )
    };

    unsafe { FreeLibrary(module) };

    if ret == 0 {
        return "unknown".to_string();
    }

    let msg = unsafe { msg.assume_init() };

    let ret = unsafe { pwstr_to_string(msg) };
    unsafe { LocalFree(msg as HLOCAL) };
    ret
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse()?;

    let mut err: i32 = if args.error.starts_with("0x") {
        i64::from_str_radix(&args.error[2..], 16)? as _
    } else {
        args.error.parse()?
    };

    if err == 0 {
        return Ok(());
    }

    if args.nt_status {
        match unsafe { ntapi::ntrtl::RtlNtStatusToDosError(err) } {
            ERROR_MR_MID_NOT_FOUND => {}
            dos => err = dos as i32,
        }
    }

    let error = format_error(err);

    match &args.error.get(..2) {
        Some("0x") | Some("0X") => println!("{} ({}): {}", args.error, err, error),
        _ => println!(
            "{} ({:#08x}): {}",
            args.error,
            args.error.parse::<i32>()?,
            error
        ),
    }

    Ok(())
}
