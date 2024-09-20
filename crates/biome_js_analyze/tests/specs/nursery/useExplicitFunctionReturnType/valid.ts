/* should not generate diagnostics */
function test(): void {
  return;
}

var fn = function (): number {
  return 1;
};

var arrowFn = (): string => 'test';

class Test {
  constructor() {}
  get prop(): number {
    return 1;
  }
  set prop() {}
  method(): void {
    return;
  }
  arrow = (): string => 'arrow';
}

const obj = {
	method(): string {
		return "test"
	}
}

const obj = {
  get method(): string {
    return "test"
  },
};
