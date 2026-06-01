Name:           hwatch
Version:        0.4.2
Release:        6%{?dist}
Summary:        Modern watch replacement with history and diff views
URL:            https://github.com/blacknon/hwatch/
# Output of %%{cargo_license_summary}
# (Apache-2.0 OR MIT) AND BSD-3-Clause
# (MIT OR Apache-2.0) AND Unicode-DFS-2016
# 0BSD OR MIT OR Apache-2.0
# Apache-2.0
# Apache-2.0 OR BSL-1.0
# Apache-2.0 OR MIT
# Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT
# ISC
# MIT
# MIT OR Zlib OR Apache-2.0
# Unlicense OR MIT
# Zlib
License:        %{shrink:
                MIT AND
                Apache-2.0 AND
                ISC AND
                BSD-3-Clause AND
                Unicode-DFS-2016 AND
                Zlib AND
                (Apache-2.0 OR MIT) AND
                (Apache-2.0 OR BSL-1.0) AND
                (Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT) AND
                (MIT OR Zlib OR Apache-2.0) AND
                (Unlicense OR MIT) AND
                (0BSD OR MIT OR Apache-2.0)
                }
Source0:        https://github.com/blacknon/hwatch/releases/download/%{version}/%{name}-%{version}.tar.gz

%bcond_without check

BuildRequires:  bash-completion
BuildRequires:  cargo-rpm-macros

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
%cargo_license_summary -a
# Keep a concrete dependency license manifest in the package, similar to helix.
/usr/bin/cargo2rpm --path Cargo.toml license-breakdown --all-features > LICENSE.dependencies
test -s LICENSE.dependencies

%install
install -D -m 644 man/hwatch.1 %{buildroot}%{_mandir}/man1/%{name}.1
install -D -m 644 completion/bash/%{name}-completion.bash %{buildroot}%{_datadir}/bash-completion/completions/%{name}
install -D -m 644 completion/fish/%{name}.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/%{name}.fish
install -D -m 644 completion/zsh/_%{name} %{buildroot}%{_datadir}/zsh/site-functions/_%{name}
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%check
%if %{with check}
/usr/bin/env CARGO_HOME=.cargo RUSTC_BOOTSTRAP=1 /usr/bin/cargo test -j%{_smp_build_ncpus} -Z avoid-dev-deps --profile rpm --no-fail-fast --all-features -- --skip test_exec_command_with_force_color_stdout_is_tty --skip test_exec_command_with_force_color_stdin_is_tty
%endif

%files
%license LICENSE LICENSE.dependencies
%doc README.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{_datadir}/bash-completion/completions/%{name}
%{_datadir}/fish/vendor_completions.d/%{name}.fish
%{_datadir}/zsh/site-functions/_%{name}

%changelog
* Mon Jun 01 2026 blacknon <blacknon@orebibou.com> - 0.4.2-6
- Update the package to follow current Fedora Rust packaging guidance more closely.
- Rewrite the License expression to preserve OR operators for bundled Rust dependencies.
- Generate LICENSE.dependencies explicitly so it is shipped reliably in the package.

* Sat May 30 2026 blacknon <blacknon@orebibou.com> - 0.4.2-4
- Remove unnecessary gcc BuildRequires.
- Replace rust-packaging with cargo-rpm-macros.
- Use plain install instead of %%cargo_install.
- Fix skipped test argument passing in %%check.
- Update the License field for statically linked Rust dependencies.

* Thu May 28 2026 blacknon <blacknon@orebibou.com> - 0.4.2-3
- Align the package with the current Fedora Rust packaging workflow.
- Update the License field for statically linked Rust dependencies.
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
