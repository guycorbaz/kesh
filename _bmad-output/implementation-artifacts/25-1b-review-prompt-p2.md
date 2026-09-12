# Prompt — passe 2 de `bmad-code-review`, Story 25-1b

*Versionné le 2026-09-12. Lentille unique (Opus 5), contexte frais, après la remédiation de la
passe 1.*

Tu es un **relecteur de code adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1b-trous-alimentation`.

## ⛔ Ton périmètre : la REMÉDIATION de la passe 1

Lis **`git show c7840f65`** — et le commit qui suit. *Dans ce dépôt, la sévérité ne stagne pas :
elle se déplace vers ce qu'on vient d'écrire.* Sept findings y ont été traités ; interroge-les.

1. **La garde contre les ALIAS** (`tests/audit_route_registry.rs`). Elle échoue si deux routes
   partagent `(verbe, handler)`. ⚠️ **Peut-elle rougir À TORT ?** Existe-t-il un cas légitime où
   deux routes partagent un handler — un alias de compatibilité, un `GET`/`POST` sur le même
   corps ? Et surtout : **ferme-t-elle vraiment le faux vert**, ou le déplace-t-elle ? Un handler
   renommé, un module réexporté, un `pub use` la contourneraient-ils ?
2. **Les six blocs d'assertions ajoutés aux tests** — réactivation, complétion, refus de
   `companies`, les trois refus de `setup`. ⚠️ **Prouvent-ils ce qu'ils prétendent ?** Cherche
   l'assertion qui passerait même si le code était faux. **Éprouve-les par la pensée** : si je
   retirais l'appel d'audit correspondant, chacun rougirait-il ?
3. **Les assertions de DELTA.** La passe 1 a établi que quatre assertions mesuraient un absolu là
   où il fallait un écart. Les corrections sont-elles justes ? En reste-t-il, **y compris hors des
   fichiers touchés par la story**, qui mesureraient la mauvaise grandeur ?
4. **Le MEDIUM documenté** — l'orphelinat de fichier si l'audit échoue après archivage. ⚠️ **La
   justification dit-elle vrai ?** L'orphelin est-il **réellement** récupérable par ré-import
   (stockage adressé par contenu) ? Le chemin de ré-import réécrit-il vraiment le même chemin, ou
   échouerait-il sur un fichier préexistant ?
5. **La terminologie corrigée** (quatre extractions, quatre conversions) : le compte est-il exact ?

## Puis, si le budget le permet

Ce que les trois lentilles de la passe 1 ont déclaré **ne pas avoir exercé** : les chemins
d'erreur exhaustifs route par route, et la compilation effective des extractions. `cargo check`,
`cargo clippy` et `cargo test` en lecture sont autorisés.

## Ce que tu rends

- **Les findings** : sévérité, fichier:ligne, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour chacun : **né de la remédiation de la passe 1**, ou d'origine ?
- **Ton verdict sur la clôture** : le critère est *plus aucun finding au-dessus de LOW*.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe. **Ne qualifie jamais de « vérifié » un point que
  tu n'as pas exécuté.**

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`add`/`stash`, `sqlx migrate`, toute écriture en base, et tout
script de `scripts/`.
