# Grammar Rules

- Break, Continue are only allowed in For, While loop
- Return value only by call `return val;`


## Parser Priority

- parse stmt based on previous token.(for -> parse_for, if -> parse_if, etc)
- parse expression in this order: 
  1. parse_or
  2. parse_and
  3. parse_equlity(==, !=)
  4. parse_compration(>, >=, <. <=).
  5. parse_term(+, -)
  6. parse_factor(*, /, %).
  7. parse_unary(+, -, !)
  8. parse_postfix(incre: ++, decre:--, fn_call),
  9. parse_primary(literal, ident, null, group, keyword->builtin)
