# Story 23.1 : Le socle — deux gardes, parce qu'il y a deux silences

## Status

split

**Découpée le 2026-08-19 en 23-1a + 23-1b**, arbitrage de Guy, après trois passes de
`bmad-create-story validate`.

⚠️ **Le corps de cette fiche est volontairement vide.** Il ne reste que les pointeurs vers les deux
moitiés — c'est la définition du statut `split` dans `sprint-status.yaml`, et elle a été écrite
parce que le précédent **17-2** avait laissé un corps complet derrière lui, ce qu'il a fallu quatre
passes pour démêler. Tout ce que cette fiche contenait vit désormais dans l'une des deux moitiés,
**identifiants de décisions et de critères inchangés**.

| moitié | objet | fiche |
|---|---|---|
| **23-1a** | Le **mécanisme** : les deux gardes, l'extracteur et son test, les 8 préfixes dynamiques, les deux allowlists, les bornes anti-test-muet. **Aucune traduction, aucun catalogue touché** — les allowlists naissent pleines, à 317 clés. | [`23-1a-mecanisme-gardes-i18n.md`](23-1a-mecanisme-gardes-i18n.md) |
| **23-1b** | Le **pilote** : le moissonneur, les 20 clés de `contacts` dans les quatre locales, le renommage de `delete`, la promotion de trois termes au glossaire, le registre d'adresse. **Dépend du merge de la 23-1a** — elle décrémente ses allowlists. | [`23-1b-pilote-contacts-glossaire.md`](23-1b-pilote-contacts-glossaire.md) |

## Pourquoi le découpage

**Trend des trois passes** : `2C/1H/8M/3L` (Sonnet) → `0C/4H/6M/2L` (Haiku) → `0C/2H/10M/6L` (Opus).
Le **plafond de sévérité stagne à HIGH** et le volume au-dessus de LOW ne décroît pas — le critère
`N+1 ≥ N` de la § *Règle de splitting préventif* est atteint.

**Le motif n'est pas le nombre de modules touchés** (quatre, sous le seuil de cinq), mais le fait
que les findings se répartissent en **deux familles qui ne se relisent pas avec la même lentille** :
le *mécanisme* — extracteur, motifs dynamiques, ensembles ouverts, allowlists, orphelines — et les
*comptes rendus* — glossaire, attestations, décomptes, commandes de recompte. Sur la passe 3,
presque tous les MEDIUM de la seconde famille étaient des **résidus laissés par les patches des
passes 1 et 2**.

C'est le motif exact du split 22-2a / 22-2b, que la rétrospective de l'Epic 22 valide.

**L'historique complet des trois passes** — findings, réfutations au ground-truth, patches — est au
Change Log de la **23-1a**. Il n'est pas dupliqué : deux copies d'un même récit divergent, et c'est
précisément le défaut que ces passes ont documenté quatre fois.

[#316]: https://github.com/guycorbaz/kesh/issues/316
[#283]: https://github.com/guycorbaz/kesh/issues/283
