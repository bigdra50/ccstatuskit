# Release automation renders the placeholders below, attaches the rendered
# formula to each GitHub Release, and pushes it to
# bigdra50/homebrew-tap (Formula/ccstatuskit.rb) when HOMEBREW_TAP_DEPLOY_KEY is set.
class Ccstatuskit < Formula
  desc "Modular statusline kit for Claude Code"
  homepage "https://github.com/bigdra50/ccstatuskit"
  version "__VERSION__"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/bigdra50/ccstatuskit/releases/download/v__VERSION__/ccstatuskit-aarch64-apple-darwin.tar.gz"
      sha256 "__OSX_ARM64_SHA256__"
    else
      url "https://github.com/bigdra50/ccstatuskit/releases/download/v__VERSION__/ccstatuskit-x86_64-apple-darwin.tar.gz"
      sha256 "__OSX_X64_SHA256__"
    end
  end

  on_linux do
    url "https://github.com/bigdra50/ccstatuskit/releases/download/v__VERSION__/ccstatuskit-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "__LINUX_X64_SHA256__"
  end

  def install
    bin.install "ccstatuskit"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ccstatuskit --version")
  end
end
