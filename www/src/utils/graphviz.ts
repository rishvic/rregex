const DOTSTR_LAST_CURLY_BRACKET_RE = /}\n$/;
const DOTSTR_BASE_LABEL_RE = /\[ label = "(\d+)" \]/g;

export function makeDotStrProper(
  dotStr: string,
  start: number,
  finStates: number[]
) {
  let properStr = dotStr.replace(
    DOTSTR_LAST_CURLY_BRACKET_RE,
    '    rankdir = LR\n}\n'
  );

  finStates.forEach(node => {
    properStr = properStr.replace(
      `[ label = "${node}" ]`,
      `[ label = "${node}" shape = doublecircle ]`
    );
  });

  properStr = properStr
    .replace(`[ label = "${start}"`, `[ label = "${start}, s"`)
    .replaceAll(DOTSTR_BASE_LABEL_RE, '[ label = "$1" shape = circle ]');

  return properStr;
}
