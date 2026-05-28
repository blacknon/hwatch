Name:           hwatch
Version:        0.4.2
Release:        3%{?dist}
Summary:        Modern watch replacement with history and diff views
URL:            https://github.com/blacknon/hwatch/
License:        MIT
Source0:        https://github.com/blacknon/hwatch/releases/download/%{version}/%{name}-%{version}.tar.gz

%bcond_without check

BuildRequires:  bash-completion
BuildRequires:  gcc
BuildRequires:  rust-packaging

%description
hwatch is an interactive terminal application similar to watch.
It records command output over time, lets users inspect history, view
differences between runs, export logs, and optionally trigger follow-up
commands when output changes.

%generate_buildrequires
%if %{with check}
%cargo_generate_buildrequires -a -t
%else
%cargo_generate_buildrequires -a
%endif

%prep
%autosetup -n %{name}-%{version}
%cargo_prep

%build
%cargo_build -a

%install
install -D -m 644 man/hwatch.1 %{buildroot}%{_mandir}/man1/%{name}.1
install -D -m 644 completion/bash/%{name}-completion.bash %{buildroot}%{_datadir}/bash-completion/completions/%{name}
install -D -m 644 completion/fish/%{name}.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/%{name}.fish
install -D -m 644 completion/zsh/_%{name} %{buildroot}%{_datadir}/zsh/site-functions/_%{name}
%cargo_install -a

%check
%if %{with check}
%cargo_test -a -- -- \
  --skip test_exec_command_with_force_color_stdout_is_tty \
  --skip test_exec_command_with_force_color_stdin_is_tty
%endif

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{_datadir}/bash-completion/completions/%{name}
%{_datadir}/fish/vendor_completions.d/%{name}.fish
%{_datadir}/zsh/site-functions/_%{name}

%changelog
* Thu May 28 2026 blacknon <blacknon@orebibou.com> - 0.4.2-3
- Align the package with the current Fedora Rust packaging workflow.
- Generate test build requirements with cargo macros and use %cargo_test.
- Remove local monkey-patching of packaged crates and dev-dependency edits.

* Wed Apr 29 2026 blacknon <blacknon@orebibou.com> - 0.4.2-2
- Use a fixed GitHub release asset for Source0 to avoid archive checksum drift.

* Tue Apr 28 2026 blacknon <blacknon@orebibou.com> - 0.4.2-1
- Prepare the package for Fedora review.
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
