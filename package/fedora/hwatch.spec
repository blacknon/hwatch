Name:           hwatch
Version:        0.4.2
Release:        10%{?dist}
Summary:        Modern watch replacement with history and diff views
URL:            https://github.com/blacknon/hwatch/

# Output of %%{cargo_license_summary -a}
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

%bcond check 1

BuildRequires:  cargo-rpm-macros

%description
hwatch is an interactive terminal application similar to watch.
It records command output over time, lets users inspect history, view
differences between runs, export logs, and optionally trigger follow-up
commands when output changes.

%generate_buildrequires
%cargo_generate_buildrequires -a

%prep
%autosetup -n %{name}-%{version}
%cargo_prep

%build
%cargo_build -a
%{cargo_license_summary -a}
%{cargo_license -a} > LICENSE.dependencies

%install
install -Dpm 0644 man/hwatch.1 \
    -t %{buildroot}%{_mandir}/man1/

install -Dpm 0644 completion/bash/%{name}-completion.bash \
    %{buildroot}%{bash_completions_dir}/%{name}

install -Dpm 0644 completion/fish/%{name}.fish \
    -t %{buildroot}%{fish_completions_dir}/

install -Dpm 0644 completion/zsh/_%{name} \
    -t %{buildroot}%{zsh_completions_dir}/

install -Dpm 0755 target/release/%{name} \
    -t %{buildroot}%{_bindir}/

%check
%if %{with check}
# Skip TTY-sensitive tests because the Fedora build environment does not
# provide a real interactive terminal for them.
%cargo_test -a -- -- --skip test_exec_command_with_force_color_stdout_is_tty --skip test_exec_command_with_force_color_stdin_is_tty
%endif

%files
%license LICENSE LICENSE.dependencies
%doc README.md
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{bash_completions_dir}/%{name}
%{fish_completions_dir}/%{name}.fish
%{zsh_completions_dir}/_%{name}
