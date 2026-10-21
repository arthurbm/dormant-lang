# Gramática do Dormant (v1)

Notação EBNF: `a?` opcional, `a*` zero ou mais, `a | b` alternativa, `( … )` agrupa, `"x"` terminal literal. Todo programa aceito é TypeScript válido.

## Microssintaxe (tokens)

```ebnf
ident      ::= [A-Za-z_] [A-Za-z0-9_]*          (* exceto palavras reservadas *)
int        ::= "0" | [1-9] [0-9]*
float      ::= ( "0" | [1-9] [0-9]* ) "." [0-9]+
string     ::= '"' ( [^"\\\r\n] | "\\" . )* '"'
whitespace ::= [ \t\r\n]+                         (* descartado *)
comment    ::= "//" [^\n]*                        (* descartado *)
```

Palavras reservadas: `async await const else export false function if let return true while`.

## Sintaxe

```ebnf
Program      ::= Function* EOF
Function     ::= "export"? "async"? "function" ident "(" Params? ")" ":" Type FunctionBody
Params       ::= Param ( "," Param )*
Param        ::= ident ":" Type
Type         ::= "Promise" "<" Type ">"
               | ident ( "[" "]" )*
FunctionBody ::= "{" Directive? Stmt* "}"
Directive    ::= ( '"use workflow"' | '"use step"' ) ";"
Block        ::= "{" Stmt* "}"

Stmt         ::= VarDecl | Assign | If | While | Return | ExprStmt
VarDecl      ::= ( "let" | "const" ) ident ( ":" Type )? "=" Value ";"
Assign       ::= ident "=" Value ";"
ExprStmt     ::= Value ";"
Value        ::= "await"? Expr
If           ::= "if" "(" Expr ")" Block ( "else" ( Block | If ) )?
While        ::= "while" "(" Expr ")" Block
Return       ::= "return" Expr? ";"

Expr         ::= Or
Or           ::= And ( "||" And )*
And          ::= Eq ( "&&" Eq )*
Eq           ::= Rel ( ( "===" | "!==" ) Rel )*
Rel          ::= Add ( ( "<" | "<=" | ">" | ">=" ) Add )*
Add          ::= Mul ( ( "+" | "-" ) Mul )*
Mul          ::= Unary ( ( "*" | "/" | "%" ) Unary )*
Unary        ::= ( "!" | "-" ) Unary | Postfix
Postfix      ::= Primary ( "(" Args? ")" | "[" Expr "]" | "." ident )*
Args         ::= Expr ( "," Expr )*
Primary      ::= int | float | string | "true" | "false" | ident
               | "[" Args? "]" | "(" Expr ")"
```

## Decisões

1. **Chaves obrigatórias** em `if`/`while`: o `else` só vem depois de `}`, então não há "else pendurado" (ambiguidade resolvida mudando a construção, EaC §3.2.3).
2. **Precedência na gramática**: um não terminal por nível, de `||` (agrupa por último) a `Postfix` (agrupa primeiro); `X ( op X )*` dá associatividade à esquerda.
3. **`await` só no começo de um comando ou logo depois de `=`**, nunca dentro de expressão; o verificador barra `await` dentro de `if`/`while` e de algo que não seja Step ou `sleep` ([ADR 0004](adr/0004-v1-scope-cuts.md)).
4. **Nomes de tipo são `ident`**: `int`, `float`, `boolean`, `string`, `void` não são palavras reservadas; quem decide se um nome é tipo é o verificador (`int`/`float` vêm do prelúdio).
5. **Parser estrito**: `;` obrigatório (sem inserção automática) e `EOF` no fim; sobra de entrada é erro de sintaxe.
