use std::ffi::OsString;
use std::io::Write;
use std::time::Duration;

use failure::err_msg;
use failure::Error;
use subprocess::Popen;
use subprocess::PopenConfig;
use subprocess::Redirection;
use tempfile::NamedTempFile;

pub fn qalc(input: &str) -> Result<String, Error> {
    let mut temp = NamedTempFile::new()?;

    let input = input.replace(" ;; ", "\n");

    writeln!(&mut temp, "{}", input)?;
    let path = temp.into_temp_path();

    let mut child = Popen::create(
        // TODO: very ugly coercing here
        &[
            OsString::from("qalc").as_os_str(),
            OsString::from("-f").as_os_str(),
            path.as_os_str(),
        ],
        PopenConfig {
            stdout: Redirection::Pipe,
            stderr: Redirection::Merge,
            ..Default::default()
        },
    )?;

    if let Some(_exit) = child.wait_timeout(Duration::from_secs(1))? {
        let (output, _) = child.communicate(None)?;
        Ok(output.ok_or_else(|| err_msg("output requested"))?)
    } else {
        child.kill()?;
        bail!("timeout, kill attempted");
    }
}
