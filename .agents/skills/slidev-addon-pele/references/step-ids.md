# Step ids beyond flowcharts and sequences

For these types a step names the `data-id`s Pele draws on the diagram's parts. Every part is nameable, edges included, and an edge shows only at the steps that name it (there is no automatic edge rule). Naming a part drawn around others (a kanban column, a journey section, a cynefin domain) names everything inside it too, and a container stays in view while any part inside it is active.

The quickest way to the exact ids: write a step with a made-up id, run the deck, and read the `[pele]` line, which lists every id of that diagram in drawing order.

| type          | ids                                                                                                                   | example                     |
| ------------- | --------------------------------------------------------------------------------------------------------------------- | --------------------------- |
| class         | class names; relations `id_<A>_<B>_<n>`                                                                               | `Animal`, `id_Animal_Dog_1` |
| state         | state ids, `root_start`, `root_end`; transitions `edge<n>` from 0                                                     | `Idle`, `edge0`             |
| er            | entity names; relations are long generated ids                                                                        | `CUSTOMER`                  |
| gantt         | task ids (`:a1,`), section names                                                                                      | `a1`, `Build`               |
| pie           | slice labels                                                                                                          | `Dogs`                      |
| mindmap       | node ids (the word before the brackets, else the text)                                                                | `root`, `Render`            |
| timeline      | period text, section names                                                                                            | `2002`                      |
| journey       | task text, section names, actors                                                                                      | `Make tea`, `Work`          |
| kanban        | column ids, card ids                                                                                                  | `todo`, `t1`                |
| gitGraph      | branch names, commit ids (`commit id: "init"`)                                                                        | `main`, `init`              |
| xychart       | series titles (`bar "Orders" [..]`); untitled series have no ids                                                      | `Orders`                    |
| quadrantChart | quadrants `1`–`4`, point names                                                                                        | `Campaign A`                |
| architecture  | service and group ids; edges `<a>-<b>`                                                                                | `db`, `api`, `db-server`    |
| c4            | element aliases; relations `<from>-<to>`                                                                              | `user`, `user-app`          |
| block         | block ids; edges `<n>-<a>-<b>`                                                                                        | `a`                         |
| requirement   | requirement and element names; relations `<src>-<dst>-<n>`                                                            | `test_req`                  |
| sankey        | node names                                                                                                            | `A`                         |
| radar         | curve ids, axis ids                                                                                                   | `x`, `a`                    |
| treemap       | section and leaf names                                                                                                | `Root`                      |
| packet        | bit ranges                                                                                                            | `0-15`                      |
| railroad      | rule names                                                                                                            | `word`                      |
| treeView      | file and folder names (folders without `/`)                                                                           | `src`                       |
| others        | agentflow and swimlane step like flowcharts; usecase, venn, cynefin, wardley, ishikawa and eventmodeling by drawn ids |                             |

`info` diagrams have no parts to step.

An id written in a step may hold only letters, digits, `_`, `-` and spaces (the characters Slidev allows in code line ranges). Parts whose id holds anything else (`index.js`, a sankey link `A->B`, a venn region `A|B`) cannot be named on their own: name the part around them, or use `*`.
