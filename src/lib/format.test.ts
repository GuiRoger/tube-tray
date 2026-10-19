import { describe, expect, it } from "vitest";
import { fileName, formatEta, formatPercent, formatRemaining, formatSize, formatSpeed, looksLikeYoutube } from "./format";

describe("format", () => {
  it("speed", () => {
    expect(formatSpeed(null)).toBe("");
    expect(formatSpeed(512 * 1024)).toBe("512 KB/s");
    expect(formatSpeed(3.25 * 1024 * 1024)).toBe("3,3 MB/s");
  });

  it("eta", () => {
    expect(formatEta(null)).toBe("");
    expect(formatEta(12)).toBe("0:12");
    expect(formatEta(125)).toBe("2:05");
    expect(formatEta(3725)).toBe("1:02:05");
  });

  it("percent", () => {
    expect(formatPercent(42.9)).toBe("42%");
    expect(formatPercent(140)).toBe("100%");
  });

  it("youtube links", () => {
    expect(looksLikeYoutube("https://youtu.be/dQw4w9WgXcQ")).toBe(true);
    expect(looksLikeYoutube("https://www.youtube.com/watch?v=dQw4w9WgXcQ")).toBe(true);
    expect(looksLikeYoutube("youtube.com/shorts/abc")).toBe(true);
    expect(looksLikeYoutube("https://vimeo.com/1")).toBe(false);
    expect(looksLikeYoutube("ola")).toBe(false);
  });

  it("file name", () => {
    expect(fileName("C:\\Users\\gui\\Downloads\\TubeTray\\a [x].mp4")).toBe("a [x].mp4");
    expect(fileName("/Users/gui/Downloads/a.mp4")).toBe("a.mp4");
  });
});

describe("remaining & size", () => {
  it("remaining", () => {
    expect(formatRemaining(18)).toBe("18s restantes");
    expect(formatRemaining(130)).toBe("2 min restantes");
    expect(formatRemaining(3900)).toBe("1 h 5 min restantes");
    expect(formatRemaining(null)).toBe("");
  });
  it("size", () => {
    expect(formatSize(186 * 1024 * 1024)).toBe("186 MB");
    expect(formatSize(1.25 * 1024 ** 3)).toBe("1,3 GB");
    expect(formatSize(5000)).toBe("5 KB");
    expect(formatSize(null)).toBe("");
  });
});
