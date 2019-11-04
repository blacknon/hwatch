%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: hwatch
Summary: alternative watch command.
Version: 0.1.1
Release: 1
License: MIT License
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
AutoReqProv:    no

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root
Requires: ncurses
Requires: ncurses-base
Requires: ncurses-libs
Requires: ncurses-devel

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
