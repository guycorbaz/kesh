# Prompt — passe 7 CIBLÉE de `bmad-create-story validate`, Story 25-1b

*Versionné le 2026-09-12. Lentille unique (Haiku 4.5), contexte frais. **Passe ciblée** sur le seul
commit de la remédiation précédente. Vocation : **clore la boucle** si elle est propre.*

Tu es un **valideur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, spec
`_bmad-output/implementation-artifacts/25-1b-trous-alimentation.md`.

## ⛔ Ton périmètre : `git show 1f02097f`, et rien d'autre

Quatre corrections y ont été portées. Relis-les, **et surtout ce qu'elles auraient dû entraîner
ailleurs** — c'est ce qui a été pris en défaut à chacune des deux passes précédentes.

1. **T10 liste désormais HUIT sites.** Vérifie que la liste est complète et exacte — les huit
   numéros de ligne, au sol, dans les `.tex`. **Et vérifie qu'aucun AUTRE endroit du document ne
   compte encore sept.**
2. **« six … cinq autres … trois … = 14 ».** Recompte **depuis le tableau des décisions**, ligne
   par ligne, et dis si la somme fait bien quatorze.
3. **Le registre porte 108 entrées** — 105 de `lib.rs` plus 3 de `routes/test_endpoints.rs` —
   quand l'inventaire, lui, porte sur 105. **Les deux nombres coexistent-ils sans se contredire
   dans le document ?** Un lecteur peut-il savoir lequel s'applique où ?
4. **La reformulation du relevé des conventions** (qui disait « les 73 sites en place » et nomme
   désormais son objet). Est-elle exacte ?

## La question qui décide de la clôture

⛔ **Reste-t-il, dans ce document, un énoncé dont la valeur est juste mais dont l'OBJET est
faux ?** C'est ce que la remédiation vient de corriger une fois, et six passes l'avaient laissé
passer. Interroge **ce que chaque nombre compte**, pas sa valeur.

## Ce que tu rends

- **Les findings** : sévérité, endroit exact, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour chacun : **né du commit `1f02097f`**, ou d'origine ?
- **Ton verdict sur la clôture**, franchement : le critère est *plus aucun finding au-dessus de
  LOW*. Si c'est propre, dis-le ; si ça ne l'est pas, dis pourquoi.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste **ne clôt rien**. **Ne qualifie jamais de « vérifié » un point que tu n'as
  pas exécuté** — dire « je ne l'ai pas fait » est un rapport utile, l'affirmer à tort ne l'est pas.

## Discipline imposée par ce dépôt

⛔ **Tout finding `CRITICAL` ou `HIGH` qui affirme l'ABSENCE d'un code attendu ou la PRÉSENCE d'un
anti-pattern doit être établi par un `grep -nF` (fixed-string) cité dans ton rapport.**

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`add`/`stash`, `sqlx migrate`, toute écriture en base, et tout
script de `scripts/`. `cargo check` et `cargo clippy` sont autorisés — rien d'autre.
