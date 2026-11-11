import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const source = readFileSync(new URL("../defaults/mods/menu/main.js", import.meta.url), "utf8")
  .replace(/^import .*;\r?\n/gm, "")
  .split("export default class Menu")[0];
function items(detached: boolean) {
  return new Function("spaces", "terminals", `${source}; return items;`)(
    () => [{ id: "space-a", detached }], () => [],
  )({ tab: "space", target: { id: "space-a" } }, null) as [string, string, object][];
}

test("space menu offers native detach without using space.close", () => {
  expect(items(false)).toContainEqual(["Open in new window", "space.popout", { id: "space-a" }]);
  expect(items(false).some((i) => i[1] === "space.dock")).toBe(false);
});

test("detached space menu retains raise and offers a non-destructive return", () => {
  expect(items(true)).toContainEqual(["Open in new window", "space.popout", { id: "space-a" }]);
  expect(items(true)).toContainEqual(["Return to main window", "space.dock", { id: "space-a" }]);
  expect(items(true)).toContainEqual(["Close", "space.close", { id: "space-a" }]);
});
