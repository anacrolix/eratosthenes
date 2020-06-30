actor Main
  new create(env: Env) =>
    let chain = DivisibleFilter(2, Printer(env), env)
    let g = Generator(1_000_000, chain)

actor Generator
  var _i: I = 2
  let _max: I
  let _next: Next
  new create(max: I, next: Next) =>
    _next = next
    _max = max
    emit()
  be emit() =>
    if _i <= _max then
      _next(_i)
      _i = _i + 1
      emit()
    end

actor Printer
  let _env: Env
  new create(env: Env) =>
    _env = env
  be apply(i: I) =>
    _env.out.print(i.string())

type I is U64

interface tag Next
  be apply(i: I)

actor DivisibleFilter
  let _div: I
  var _next: Next tag
  var _is_last: Bool
  let _env: Env
  new create(div: I, next: Next tag, env: Env) =>
    _div = div
    _next = next
    _is_last = true
    _env = env
  be apply(i: I) =>
    // _env.out.print(_div.string()+" got "+i.string())
    if (i % _div) == 0 then
      return
    end
    _next.apply(i)
    if _is_last then
      _next = DivisibleFilter(i, _next, _env)
      _is_last = false
    end
