# Prompt — passe 1 de `bmad-create-story validate`, Story 25-2-b-zero

*Versionné le 2026-09-19. Une lentille en contexte frais (Sonnet), orthogonale à l'auteur de la
spec (Opus 5). Une seule lentille : la story touche une fonction et ses tests.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-2-b-zero-verrou-periode-suppression.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-2-b-zero-verrou-periode-suppression` (tirée de `main`
au `44c6842f`). Ta mission n'est pas d'approuver : c'est de **trouver ce qui ferait échouer, dévier
ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole.** Chaque affirmation — chemin, ligne, nom de fonction, ordre des
étapes, comportement d'une route, contenu d'un manuel — se **vérifie dans le code actuel**.

⚠️ **Ne conteste PAS l'arbitrage** : la garde sort de la 25-2-b en story propre, livrée d'abord.

## Sources

- l'issue **#443** (`gh issue view 443`), la 24-4c (#380, fermée) et `crates/kesh-api/tests/period_lock_e2e.rs` ;
- `_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md` (d'où elle est détachée) ;
- `CLAUDE.md` — § *Propagation post-patch*, § *Un gate laisse la base piégée*, § *Le prompt d'une
  passe doit NOMMER le manuel* ;
- **le code**, qui prime.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Exactitude.** Chaque `fichier:ligne`, chaque nom, l'ordre annoncé des étapes de `delete_in_tx`.
2. **Le défaut est-il réel et complet ?** Existe-t-il un AUTRE chemin qui fait disparaître ou
   modifie une écriture d'une période verrouillée — suppression d'un avoir, d'une facture
   fournisseur, d'un règlement, d'un rapprochement, import de sauvegarde, réinitialisation démo,
   `ON DELETE CASCADE` sur `journal_entry_lines` ou ailleurs, mise à jour de lignes ? Inventorie les
   sites qui **suppriment ou modifient** des lignes de `journal_entries` / `journal_entry_lines`
   hors `delete_in_tx`, et dis pour chacun s'il est hors périmètre à bon droit.
3. **L'ordre et le contrat (AC 3).** L'ordre annoncé est-il le bon, et la route gelée garde-t-elle
   exactement son contrat actuel ? `delete_by_id` fait-il autre chose avant `delete_in_tx` ?
4. **Les verrous (AC 4).** L'argument « lecture sans verrou, comme à la création » tient-il dans
   `delete_in_tx`, dont l'ordre des verrous n'est pas celui de la création ? Une course avec la pose
   d'une borne produit-elle un résultat faux, ou seulement l'écart d'un instant que la création
   accepte déjà ?
5. **Les tests (AC 6).** Chaque test tranche-t-il ? Le montage (pose puis levée de la borne,
   dates antérieures à la borne, borne antérieure à aujourd'hui) est-il constructible avec les
   helpers existants — `companies::lock_books` et sa levée, ceux de `period_lock_e2e.rs` ? Une
   mutation produirait-elle une erreur de compilation plutôt qu'un échec d'assertion ? Un résidu en
   base est-il possible ?
6. **Les manuels (AC 7).** Les deux sites cités, leur PDF aplati
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), et tout autre site — manuels, `README.md`,
   `website/`, `docs/api-external.md` — qui parle du verrou de période ou de la suppression d'une
   facture validée et que la story rendrait faux ou laisserait incomplet.
7. **Cohérence et périmètre.** La story ne déborde-t-elle pas sur la 25-2-b, et ne lui laisse-t-elle
   rien d'implicite ?

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et ce qu'il faut changer. Pour un scénario, montre que son
  état de départ est atteignable. Un reproche au code de ne pas encore faire ce que la story
  prescrit n'est **pas** un défaut de la spec.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une
base persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `sqlx migrate`
sur `kesh` ou `kesh_e2e`, `cargo test`/`cargo nextest`. Lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`, `pdftotext` et `cargo check` sont autorisés.
