# Prompt — passe 4 de `bmad-create-story validate`, Story 25-1b

*Versionné le 2026-09-12. Lentille unique (Haiku 4.5), contexte frais, après la remédiation
de la passe 3.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est
`_bmad-output/implementation-artifacts/25-1b-trous-alimentation.md`, dépôt
`/home/gcorbaz/devel/kesh`.

## ⛔ Ton axe PRINCIPAL : les 73 routes réputées tracées

**Trois passes successives ont déclaré cet axe non exercé.** C'est le dernier angle mort de
l'inventaire sur lequel repose toute la story, et il ne se fermera pas tout seul. Il est
énumératif : traite-le comme tel, méthodiquement.

La spec affirme que **73 des 105 routes mutantes sont déjà tracées**. Pour chacune :

1. pars de sa déclaration dans `crates/kesh-api/src/lib.rs` ;
2. suis le handler jusqu'à l'appel de `audit_log::insert_in_tx`, **où qu'il soit** — dans la route
   ou dans le repository, parfois à trois appels de distance ;
3. et pose **la question qui compte** : *la trace part-elle sur TOUS les chemins de succès, ou
   seulement sur une BRANCHE ?* Une route dont l'audit vit dans un `if`, un `match`, une boucle ou
   après un `?` qui peut court-circuiter est une route **partiellement** tracée — et l'inventaire
   la compte comme tracée.

⚠️ **Ce que tu cherches n'est pas l'absence d'un appel** — elle se voit au grep — **mais un appel
qui ne s'exécute pas toujours.** L'inventaire a déjà trouvé **deux** cas de ce genre
(`/complete` et `/onboarding/finalize`) ; rien ne dit qu'il n'y en a pas un troisième.

**Rends un tableau** : route, où la trace est écrite (fichier:ligne), et **inconditionnelle ou
conditionnelle**. Si le budget ne permet pas les 73, dis **combien** tu en as réellement traitées
et **lesquelles** — un échantillon déclaré vaut mieux qu'un total supposé.

## Tes axes secondaires

- **Les patches de la passe 3** (`git show 95b87e0c`) : T6 et l'interdiction de poser la trace dans
  `update_step` ; le sixième site du manuel (`admin:1762`) ; la portée déclarée de « 1 partielle » ;
  les deux LOW. Sont-ils justes, et n'ont-ils rien cassé ?
- **Un SEPTIÈME site de manuel** : il y en a eu six, dont un trouvé à la passe 3 parce qu'il parlait
  d'*attribution* et non de *couverture*. Cherche une troisième question encore non posée.
- **Les décomptes** : 14 routes, 12 AC, 11 tâches. Recompte depuis la source.

## Ce que tu rends

- **Les findings** : sévérité, endroit exact, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour chacun : né d'un patch de la passe 3, ou d'origine ?
- **Le tableau des routes réellement traitées** (cf. axe principal).
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste **ne compte pas comme passe**. ⚠️ **Ne qualifie JAMAIS de « robuste » ou de
  « vérifié » un point que tu n'as pas exécuté** — dire « je ne l'ai pas fait » est un rapport
  utile, l'affirmer à tort ne l'est pas.

## Discipline imposée par ce dépôt

⛔ **Tout finding `CRITICAL` ou `HIGH` qui affirme l'ABSENCE d'un code attendu ou la PRÉSENCE d'un
anti-pattern doit être établi par un `grep -nF` (fixed-string) cité dans ton rapport.** Sans cette
preuve, il sera écarté sans discussion.

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`add`/`stash`, `sqlx migrate`, toute écriture en base, et tout script
de `scripts/`. `cargo check` et `cargo clippy` sont autorisés — rien d'autre.
