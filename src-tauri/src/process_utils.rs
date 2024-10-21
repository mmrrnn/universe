use std::{future::Future, path::Path, time::Duration};
use tokio::time;

pub fn launch_child_process(
    file_path: &Path,
    envs: Option<&std::collections::HashMap<String, String>>,
    args: &[String],
) -> Result<tokio::process::Child, anyhow::Error> {
    #[cfg(not(target_os = "windows"))]
    {
        Ok(tokio::process::Command::new(file_path)
            .args(args)
            .envs(envs.cloned().unwrap_or_default())
            .stdout(std::process::Stdio::null()) // TODO: uncomment, only for testing
            .stderr(std::process::Stdio::null()) // TODO: uncomment, only for testing
            .kill_on_drop(true)
            .spawn()?)
    }
    #[cfg(target_os = "windows")]
    {
        use crate::consts::PROCESS_CREATION_NO_WINDOW;
        Ok(tokio::process::Command::new(file_path)
            .args(args)
            .envs(envs.cloned().unwrap_or_default())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .creation_flags(PROCESS_CREATION_NO_WINDOW)
            .spawn()?)
    }
}

pub fn set_interval<F, Fut>(mut f: F, dur: Duration)
where
    F: Send + 'static + FnMut() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Create stream of intervals.
    let mut interval = time::interval(dur);

    tokio::spawn(async move {
        // Skip the first tick at 0ms.
        interval.tick().await;
        loop {
            // Wait until next tick.
            interval.tick().await;
            // Spawn a task for this tick.
            tokio::spawn(f());
        }
    });
}

// pub async fn launch_and_get_outputs(
//     file_path: &Path,
//     args: Vec<String>,
// ) -> Result<Vec<u8>, anyhow::Error> {
//     #[cfg(not(target_os = "windows"))]
//     {
//         let child = tokio::process::Command::new(file_path)
//             .args(args)
//             .stdout(std::process::Stdio::piped())
//             .kill_on_drop(true)
//             .spawn()?;

//         let output = child.wait_with_output().await?;
//         Ok(output.stdout.as_slice().to_vec())
//     }

//     #[cfg(target_os = "windows")]
//     {
//         use crate::consts::PROCESS_CREATION_NO_WINDOW;
//         let child = tokio::process::Command::new(file_path)
//             .args(args)
//             .stdout(std::process::Stdio::piped())
//             .kill_on_drop(true)
//             .creation_flags(PROCESS_CREATION_NO_WINDOW)
//             .spawn()?;

//         let output = child.wait_with_output().await?;
//         Ok(output.stdout.as_slice().to_vec())
//     }
// }
