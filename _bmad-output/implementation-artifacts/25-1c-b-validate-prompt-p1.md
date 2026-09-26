# Prompt — passe 1 de `bmad-create-story validate`, Story 25-1c-b

*Versionné le 2026-09-15. Deux lentilles en contexte frais (Sonnet, Haiku), orthogonales à l'auteur
de la spec (Opus 5).*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-b-journal-audit-ecran.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. Ta mission n'est pas
d'approuver : c'est de **trouver ce qui ferait échouer, dévier ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole, surtout pas cette spec.** Chaque affirmation — chemin, numéro de ligne,
nombre, comportement — se **vérifie dans le code** avant d'être retenue ou contestée.

⚠️ **Ne conteste PAS les arbitrages du Project Lead** (`epic-25-vague1-suite.md` § *Arbitrages du
2026-09-15*) ni le **contrat de la route**, fixé par la spec validée
`25-1c-a-journal-audit-route.md` (AC 5-14). Conteste la **mise en œuvre** côté écran, et la fidélité de
cette spec au contrat qu'elle consomme.

## Sources

- `25-1c-a-journal-audit-route.md` (le contrat), `epic-25-vague1-suite.md`, l'issue **#378** ;
- `CLAUDE.md` — § *E2E (Playwright)*, § *Propagation post-patch*, § *Synchroniser le planning du README*,
  § *Le prompt d'une passe doit NOMMER le manuel* ; `docs/testing.md` ; `docs/i18n-glossaire.md` ;
- **le code**, qui prime.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Exactitude factuelle.** Chaque `fichier:ligne` de la spec, vérifié.
2. **Fidélité au contrat de la 25-1c-a.** Les types de l'AC 1 correspondent-ils **champ par champ** au
   DTO de l'AC 8 de la 25-1c-a (noms, casse, nullabilité, `actorType` en `"user"`/`"api_key"`) ? Les
   paramètres, la pagination, les codes d'erreur (`RESULT_TOO_LARGE`, 400 sur `entityId` sans
   `entityType`) sont-ils consommés comme la route les définit ?
3. **Les 28 types d'entité (AC 3).** Recompte-les **toi-même** depuis `crates/*/src` (hors tests) par ta
   propre méthode. La liste est-elle exacte ? Un type manque-t-il, un code est-il mal orthographié ? Les
   replis français sont-ils cohérents avec la partie A du glossaire et les libellés déjà présents dans les
   catalogues ?
4. **Les gardes i18n (AC 7, 11, 12).** Les mises à jour prescrites de `i18n-keys.test.ts`
   (`MOTIFS_DYNAMIQUES`, `CARDINALITES`, `FAMILLES_RESOLUES`, `ATTENDU`) et de
   `i18n-libelle-en-dur.test.ts` sont-elles **exactement** celles que ces tests exigeront ? Lis la mécanique
   réelle de ces tests — par exemple : comment `error-label.ts` appelle-t-il `i18nMsg`, et que compte
   alors la garde ? `lint-i18n-ownership` accepte-t-il une feature `audit-log` et la clé `nav-audit-log` ?
5. **L'écran (AC 5-8).** La garde, le menu `comptableOnly`, les `data-testid` : chaque exigence est-elle
   décidable et testable ? L'heure locale à l'écran contre l'UTC du CSV est-elle cohérente et écrite au
   manuel ? Un état non prévu (session expirée, 403 de la route pour un rôle rétrogradé) ?
6. **Le téléchargement partagé (AC 9, 10).** Les quatre copies de `triggerDownload` sont-elles
   **identiques** ? Leur remplacement change-t-il un comportement ou un test existant ?
7. **Les manuels, le README, le glossaire (AC 13-18).** Que rend faux l'écran, que laisse-t-il vrai ?
   Contrôle le **PDF aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) ; en LaTeX le souligné
   s'écrit `\_` ; le catalogue français écrit l'apostrophe **typographique**. Cherche un site que la spec
   **ne nomme pas** — manuels, README, `website/`, en français **et** en anglais, et les clés i18n qui
   parlent d'audit.
8. **Les tests et mutations (AC 19-21).** Chaque assertion **tranche**-t-elle ? Qu'est-ce qui la rendrait
   fausse ? L'E2E est-il constructible avec les helpers existants (création d'un contact par
   `authedApiContext`, utilisateur Consultation) ?
9. **Cohérence interne, décomptes, périmètre** — recompte, ne relis pas ; rien qui touche le backend hors
   catalogues ; la § *Règle de splitting* respectée.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de
  départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été**, avec pour chaque axe
  exercé **au moins une commande réellement lancée**. Ne qualifie jamais de « vérifié » ce que tu n'as
  pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh` (il bumpe les versions Cargo),
`scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/`stash`/
`reset`/`rebase`, `npm install`, `npm run build`, `sqlx migrate`, `cargo test`/`cargo nextest`.
`npm run test:unit` et `npm run check` en lecture sont autorisés.
