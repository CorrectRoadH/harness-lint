class HarnessLint < Formula
  desc "GritQL rule ecosystem and AI feedback linter"
  homepage "https://github.com/harness-lint/harness-lint"
  version "0.1.0"

  if Hardware::CPU.arm?
    url "https://github.com/harness-lint/harness-lint/releases/download/v0.1.0/harness-lint-macos-aarch64"
    sha256 "CHANGE_ME"
  end

  def install
    bin.install "harness-lint-macos-aarch64" => "harness-lint"
  end

  test do
    system "#{bin}/harness-lint", "--help"
  end
end

