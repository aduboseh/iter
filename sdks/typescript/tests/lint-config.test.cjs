const assert = require("node:assert/strict");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");

const sdkRoot = path.resolve(__dirname, "..");
const eslintRoot = path.dirname(require.resolve("eslint/package.json"));
const eslintBin = path.join(eslintRoot, require("eslint/package.json").bin.eslint);

function lint(source, filePath = "src/lint-probe.ts", extraArgs = []) {
  const result = spawnSync(process.execPath, [
    eslintBin, "--stdin", "--stdin-filename", filePath,
    "--format", "json", "--max-warnings", "0", ...extraArgs,
  ], { cwd: sdkRoot, input: source, encoding: "utf8", timeout: 30_000 });
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  assert.ok(result.status === 0 || result.status === 1, result.stderr);
  const reports = JSON.parse(result.stdout);
  assert.equal(reports.length, 1);
  assert.equal(reports[0].fatalErrorCount, 0, result.stdout);
  return { status: result.status, report: reports[0] };
}

test("valid typed production code is linted without errors or warnings", () => {
  const { status, report } = lint("export const value: number = 1;");
  assert.equal(status, 0);
  assert.deepEqual(report.messages, []);
});

for (const filePath of ["src/lint-probe.ts", "src/lint-probe.test.ts"]) {
  test(`recommended rules reject debugger statements in ${filePath}`, () => {
    const { status, report } = lint("debugger;", filePath);
    assert.equal(status, 1);
    assert.ok(report.messages.some((message) => message.ruleId === "no-debugger"));
  });
}

for (const [source, rule] of [
  ["export const value: any = 1;", "@typescript-eslint/no-explicit-any"],
  ["export const callback: Function = () => 1;", "@typescript-eslint/no-unsafe-function-type"],
]) {
  test(`${rule} remains enforced in production`, () => {
    const { status, report } = lint(source);
    assert.equal(status, 1);
    assert.ok(report.messages.some((message) => message.ruleId === rule));
  });
}

test("only test mocks may use explicit any and Function", () => {
  const { status, report } = lint(
    "export const callback: Function = () => 1; export const client: any = { callback };",
    "src/lint-probe.test.ts",
  );
  assert.equal(status, 0);
  assert.deepEqual(report.messages, []);
});

test("warnings also fail the lint CLI", () => {
  const { status, report } = lint("debugger;", "src/lint-probe.ts", ["--rule", "no-debugger:warn"]);
  assert.equal(status, 1);
  assert.equal(report.errorCount, 0);
  assert.equal(report.warningCount, 1);
  assert.equal(report.messages[0].ruleId, "no-debugger");
});
