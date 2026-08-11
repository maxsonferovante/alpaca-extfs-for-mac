class AlpacaExtfs < Formula
  desc "Ext4 filesystem mount utility for macOS integrated with Finder and macFUSE"
  homepage "https://github.com/maxsonferovante/alpaca-extfs-for-mac"
  url "https://github.com/maxsonferovante/alpaca-extfs-for-mac/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "c19378dce305f6abd0f0b93aff2084ccaf7de616559e785c069c9c0e8e90a2c7"
  license "MIT"
  head "https://github.com/maxsonferovante/alpaca-extfs-for-mac.git", branch: "master"

  depends_on "pkg-config" => :build
  depends_on "rust" => :build
  depends_on :macos

  def install
    ENV.append_path "PKG_CONFIG_PATH", "/opt/homebrew/lib/pkgconfig"
    ENV.append_path "PKG_CONFIG_PATH", "/usr/local/lib/pkgconfig"
    ENV.append_path "PKG_CONFIG_PATH", "/Library/Filesystems/macfuse.fs/Contents/Resources/pkgconfig"
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      alpaca-extfs requires macFUSE and root permissions (sudo) to mount Ext4 filesystems.

      1. Install macFUSE cask if you haven't already:
         brew install --cask macfuse

      2. System Extension permission may be required:
         System Settings -> Privacy & Security -> Allow macFUSE extension

      3. Usage example:
         sudo alpaca-extfs /dev/rdisk4s2 /Volumes/Ext4Drive
         sudo alpaca-extfs -u /Volumes/Ext4Drive
    EOS
  end

  test do
    assert_match "Ext4 Read/Write driver for macOS", shell_output("#{bin}/alpaca-extfs --help")
  end
end
