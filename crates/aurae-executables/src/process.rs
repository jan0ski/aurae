/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use nix::sys::signal::SIGINT;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::io;
use std::ops::Deref;
use std::os::fd::OwnedFd;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};
use tracing::info;

#[derive(Debug)]
pub(crate) struct Process {
    pub inner: procfs::process::Process,
    pub pid_fd: Option<OwnedFd>,
    pub child: Option<Child>,
}
// TOOD: use states pattern to use Child when possible
impl Process {
    pub fn kill(&self) -> io::Result<ExitStatus> {
        let pid = Pid::from_raw(self.inner.pid);
        nix::sys::signal::kill(pid, SIGINT)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        // https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitpid.html
        // The waitpid() function obtains status information for process termination,
        // and optionally process stop and/or continue, from a specified subset of the child processes.
        // If pid is greater than 0, it specifies the process ID of a single child process for which status is requested.
        let exit_status = loop {
            let WaitStatus::Exited(_, exit_status) = nix::sys::wait::waitpid(pid, None)
                .map_err(|e| io::Error::from_raw_os_error(e as i32))? else {
                continue;
            };

            break exit_status;
        };

        let exit_status = ExitStatus::from_raw(exit_status);

        info!("Executable with pid {pid} exited with status {exit_status}",);

        Ok(exit_status)
    }
}

impl Deref for Process {
    type Target = procfs::process::Process;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
