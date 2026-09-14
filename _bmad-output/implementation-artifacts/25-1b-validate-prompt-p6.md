# Prompt — passe 6 CIBLÉE de `bmad-create-story validate`, Story 25-1b

*Versionné le 2026-09-12. Lentille unique (Sonnet 4.6), contexte frais. **Passe ciblée** : une
seule lentille, braquée sur **le seul commit de la remédiation précédente**.*

Tu es un **valideur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, spec
`_bmad-output/implementation-artifacts/25-1b-trous-alimentation.md`.

## ⛔ Ton périmètre : `git show 2f87b1aa`, et rien d'autre d'abord

La passe 5 a produit **neuf correctifs**. Ce sont eux — et eux seuls — qu'il s'agit de relire.
⚠️ **La passe 5 a justement trouvé qu'un correctif de la passe 3 avait laissé debout la
justification qu'il venait de vider.** Cherche la même chose ici : un patch dont le texte
environnant n'a pas suivi.

Les six objets :

1. **AC 11, le diff à DEUX VOLETS.** « Route absente du registre » porte sur `lib.rs` **et**
   `routes/test_endpoints.rs` ; « entrée disparue de `lib.rs` » ne porte que sur `lib.rs`. Est-ce
   **exact, implémentable, et suffisant** ? Une quatrième route ajoutée à `test_endpoints.rs`
   rougirait-elle réellement ? Le texte qui entoure le patch dit-il encore vrai ?
2. **AC 7, la quatorzième route.** `complete` a été ajoutée à la ligne `from_current_user`.
   ⚠️ **Est-ce le bon constructeur ?** Vérifie où `POST /imported-supplier-invoices/{id}/complete`
   est montée dans `crates/kesh-api/src/lib.rs`, et si le bloc la protège d'un jeton d'API.
   Le tableau couvre-t-il maintenant **exactement** les quatorze routes, sans doublon ni oubli ?
3. **AC 12, le huitième site** (`user-manual.tex:1733`) et l'énoncé de tête refondu (« Les manuels
   administrateur ET utilisateur… **huit** affirmations »). Les **huit** sites sont-ils exacts au
   sol, `.tex` **et** PDF aplati ? La table est-elle bien d'un seul tenant ? ⚠️ **Et cherche un
   NEUVIÈME** — la passe 5 a établi que l'inventaire se clôt par le **produit** fichiers ×
   questions, non par l'un ou l'autre. Applique-le : quelles questions déjà posées n'ont pas
   visité quels fichiers ?
4. **« SIX des quatorze chemins »** (Dev Notes). Recompte au sol, repository par repository :
   combien ouvrent **et** commitent leur propre transaction, combien n'en ouvrent **aucune**,
   combien en ont déjà une au handler. La somme fait-elle quatorze ?
5. **Les correctifs mineurs** : T4 « trois routes », T6 et `update_step_in_tx`, T7 et l'AC 7,
   la ligne vide supprimée, les aphorismes.
6. **Les décomptes** : 14 routes, 12 AC, 11 tâches, 8 sites. Recompte **chacun** depuis la source.

## Ce que tu rends

- **Les findings** : sévérité, endroit exact, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour chacun : **né d'un correctif de la passe 5**, ou d'origine ?
- **Ton verdict sur la clôture**, franchement : le critère est *plus aucun finding au-dessus de
  LOW*.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien. **Ne qualifie jamais de « vérifié » un point que tu n'as pas
  exécuté.**

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`add`/`stash`, `sqlx migrate`, toute écriture en base, et tout
script de `scripts/`. `cargo check` et `cargo clippy` sont autorisés — rien d'autre.
