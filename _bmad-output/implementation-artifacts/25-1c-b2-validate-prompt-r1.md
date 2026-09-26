# Prompt — revalidation R1 CIBLÉE de `bmad-create-story validate`, Story 25-1c-b2 (réouverte)

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Opus), orthogonale à la passe 7 (Sonnet).
Passe ciblée : la réouverture ne touche que quelques paragraphes de la fiche.*

Tu es un **chasseur de régressions** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Ton périmètre est la réouverture**, et seulement elle :
`git diff 5760bab7 HEAD -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`. Nomme
cette base dans ton rapport. Les fiches sœurs rouvertes sont `25-1c-a-journal-audit-route.md` et
`25-1c-b1-journal-audit-ecran.md` (même branche).

⛔ **Rien ne se croit sur parole.** ⚠️ **Ne conteste PAS les arbitrages du Project Lead** (filtre strict,
export non audité, vocabulaire traduit, correction de `:1796` et `:1778` dans cette story).

## Les axes, et tu déclareras lesquels tu as exercés

1. **Chaque énoncé neuf de l'AC 5 est-il vrai** au regard de la 25-1c-a rouverte (AC 8, 11, 12, 16, 17) et
   de la b1 rouverte (AC 3, 5) — langue de l'interface, filtres en listes, repli sur le code d'une action
   historique, plafond ? Un cas où l'écran rend encore une liste vide ou un refus sans explication, et que
   le manuel ne dit pas ?
2. **L'AC 6** : « ni la consultation ni l'export ne s'inscrivent au journal » est-il exact contre la 25-1c-a
   (AC 9, 14) ? Les consignes de correction de `admin-manual.tex:1796` et `:1778` sont-elles justes contre le
   `.tex` actuel (cite les lignes) et contre le code (`crates/kesh-db/src/entities/user.rs`,
   `repositories/invoices.rs` autour de 1339) ? La ligne `:1345` dit-elle bien « trois rôles » ?
3. **Propagation** : un paragraphe **non modifié** par le diff dit-il encore l'ancienne conception
   (« société indéterminée », « export tracé », `audit_log.exported`, clé `audit-log-entity-audit-log`,
   filtre d'action en champ libre, « en attente d'arbitrage ») ? Les renvois vers les AC de la 25-1c-a
   rouverte sont-ils tous justes ?
4. **L'AC 2** : la vérification des « clés neuves » est-elle exécutable telle qu'écrite, maintenant que la
   25-1c-a crée 123 clés `audit-log-*` ?
5. **Le Change Log de la réouverture** : fidèle au diff, décomptes justes.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout
`git commit`/`push`/`checkout`/`switch`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`, `git show`/`git diff`, Python en
lecture, `pdftotext` vers la sortie standard ou vers le scratchpad
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r1-b2/`.
⚠️ `grep` est ici `ugrep`, qui refuse les motifs trop complexes : utilise Python pour les extractions à
contexte.
