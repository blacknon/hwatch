Name:           hwatch
Version:        0.4.2
Release:        1%{?dist}
Summary:        A modern alternative to the 'watch' command, it records differences in execution results and allows for examination of these differences afterward.
URL:            https://github.com/blacknon/hwatch/
License:        MIT
Source0:        https://github.com/blacknon/hwatch/archive/refs/tags/%{version}.tar.gz

BuildRequires:  bash-completion
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  rust

%description
hwatch is a alternative watch command. Records the results of command execution that can display its history and differences.

Features:
* Can keep the history when the difference, occurs and check it later.
* Can check the difference in the history. The display method can be changed in real time.
* Can output the execution result as log (json format).
* Can load diffmode plugins as dynamic libraries and add custom diff rendering.
* Custom keymaps are available.
* Support ANSI color code.
* Execution result can be scroll.
* Not only as a TUI application, but also to have the differences output as standard output.
* If a difference occurs, you can have the specified command additionally executed.

%prep
%autosetup -n %{name}-%{version}

%build
export RUSTFLAGS="-C link-arg=-fuse-ld=bfd"
cargo build --release --locked --all-features

%install
install -D -m 644 man/hwatch.1 %{buildroot}%{_mandir}/man1/%{name}.1
install -D -m 644 completion/bash/%{name}-completion.bash %{buildroot}%{_datadir}/bash-completion/completions/%{name}
install -D -m 644 completion/fish/%{name}.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/%{name}.fish
install -D -m 644 completion/zsh/_%{name} %{buildroot}%{_datadir}/zsh/site-functions/_%{name}
install -D -m 755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%check
cargo test --release --locked --all-features -- \
  --skip test_exec_command_with_force_color_stdout_is_tty \
  --skip test_exec_command_with_force_color_stdin_is_tty

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{_datadir}/bash-completion/completions/%{name}
%{_datadir}/fish/vendor_completions.d/%{name}.fish
%{_datadir}/zsh/site-functions/_%{name}

%changelog
* Sat Apr 25 2026 - blacknon - 0.4.2-1
* Sat Apr 25 2026 - blacknon - 0.4.1-1
* Sun Apr 19 2026 - blacknon - 0.4.0-1
* Wed Apr 15 2026 - Danie de Jager - 0.3.20-1
* Mon Oct 20 2025 - Danie de Jager - 0.3.19-3
* Sun Jul 13 2025 - Danie de Jager - 0.3.19-2
* Wed Mar 19 2025 - blacknon - 0.3.19-1
 - [FR] add precise interval option #111
 - [FR] Pause/freeze command execution #133
 - Process freeze and terminal corruption on FreeBSD (Fixed in #178) #179
 - [FR] Disable line wrapping #182
* Fri Nov 15 2024 - blacknon - 0.3.18-1
 - fix hwatch 0.3.17 freezes in a narrow terminal  #171
 - fix hwatch 0.3.17 no longer prints blank lines. #172
* Wed Nov 13 2024 - blacknon - 0.3.17-1
 - Bugfix. Fixed the filter keyword not supporting multi-byte characters.
 - Bugfix. Fixed freezes in a narrow terminal when used with `--no-help-banner` (issue #169)
* Sun Nov 10 2024 - blacknon - 0.3.16-1
 - Bugfix an issue where the ESC key was unintentionally triggered during mouse operations on MacOS
 - Enhancement of filter (issue #124)
 - [FR] Ability to load a previously recorded log file for visualization (issue #101)
* Wed May 29 2024 - Danie de Jager - 0.3.15-1
* Mon May 13 2024 Danie de Jager - 0.3.14-2
 - strip binary
 - add bash completion
* Mon May 13 2024 Danie de Jager - 0.3.14-1
 - Initial version
