# Prompt — passe 3 de `bmad-code-review`, Story 25-1b

*Versionné le 2026-09-13. Lentille unique (Sonnet 4.6), contexte frais, après la remédiation de la
passe 2.*

Tu es un **relecteur de code adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1b-trous-alimentation`.

## ⛔ Ton périmètre : `git show 6808ab07`

Neuf findings y ont été traités. *Dans ce dépôt, la sévérité se déplace vers ce qu'on vient
d'écrire* — et la passe 2 vient d'en administrer la preuve : **trois de ses cinq findings sérieux
étaient nés de la remédiation de la passe 1**, dont une garde qui déplaçait le faux vert qu'elle
prétendait fermer.

Les six objets, par ordre de risque :

1. ⛔ **Le patch de PRODUCTION** — `routes/users.rs`, la clé `email_present` avant/après.
   ⚠️ **Est-il juste ?** Le `before` est lu où, exactement, et reflète-t-il bien l'état antérieur ?
   Et **est-il suffisant** : d'autres champs de cette route s'effacent-ils aussi par omission sans
   être tracés ? ⚠️ **Et ailleurs** — d'autres routes de la story ont-elles la même sémantique de
   remplacement sans que leur trace le dise ? *C'est ce manqué de propagation qui a produit le
   défaut.*
2. **L'inventaire des sites non lus** et la **nouvelle troncature** de `lib.rs`
   (`tests/audit_route_registry.rs`). La coupe au marqueur `// NOTE: les stories futures…`
   peut-elle rater — marqueur réécrit, déplacé, traduit ? L'assertion de garde qui l'accompagne
   suffit-elle ? ⚠️ **L'inventaire peut-il encore être contourné par une forme d'écriture
   non prévue** — `.route_service`, une macro, un `Router::new().merge(...)` ?
3. **La garde du troisième fichier** (`no_third_route_file_escapes_the_registry`). Son extraction
   des `::router()` est-elle fiable ? Peut-elle rougir à tort, ou manquer un montage réel ?
4. **Le rétablissement de l'absolu** et le **helper local** de `setup_admin_e2e.rs`. Le
   raisonnement — « ici l'absolu est exact car `truncate_all` en tête » — tient-il pour **chacun**
   des sites concernés ?
5. **Le lien trace ↔ gagnant** de la race TOCTOU : l'assertion prouve-t-elle ce qu'elle dit, ou
   passerait-elle par coïncidence sur un jeu d'un seul utilisateur ?
6. **Le manuel** : « 87 des 105 routes à verbe mutant ». Le compte et le dénominateur sont-ils
   exacts ? ⚠️ Contrôle le **PDF aplati**, et méfie-toi des apostrophes typographiques.

## Puis, si le budget le permet

Ce que les passes 1 et 2 ont **déclaré ne pas avoir exercé** : les chemins d'erreur de
`companies::update_company_contact_details`, `contact_persons::{create_person, delete_person}`,
`users::reset_password`, et la branche break-glass de `bootstrap.rs`.

## Ce que tu rends

- **Les findings** : sévérité, fichier:ligne, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour chacun : **né de la remédiation de la passe 2**, ou d'origine ?
- **Ton verdict sur la clôture** : le critère est *plus aucun finding au-dessus de LOW*.
  ⚠️ Note que la boucle ne peut de toute façon pas se clore sur cette passe, un patch de
  **production** ayant été appliqué en passe 2 — dis-le si tu le constates aussi.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe. **Ne qualifie jamais de « vérifié » un point que
  tu n'as pas exécuté.**

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`add`/`stash`, `sqlx migrate`, toute écriture en base, et tout
script de `scripts/`. `cargo check`, `cargo clippy` et `cargo test` en lecture sont autorisés.
