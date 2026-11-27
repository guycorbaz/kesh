# Prompt — revue de code P1 de la story 55-3a-justificatifs-serveur (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-3a-justificatifs-serveur** de Kesh. **Passe complète, première de la boucle.
Modèle : Opus. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-3a-justificatifs-serveur.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `174fa5385` (code et tests), `27fa832f5` (`docs/api-external.md`).
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-3a-justificatifs-serveur/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : chaque critère de la fiche, un par un, au code et aux tests.
- **Le contrat** (D9) : `hasDocuments` au journal et au grand livre (`kesh-report`), la route
  `GET /journal-entries/{id}/documents` (`kesh-api/src/routes/journal_entries.rs`, `services/ecritures.rs`) ; une seule
  définition des rattachements (DRY) entre le booléen et la liste ; règles de rattachement (facture, avoir, document
  fournisseur, règlements, écritures annulées ou contre-passées).
- **Sécurité** : société de l'écriture (IDOR), rôles, clés d'API (portées), écriture d'une autre société ; aucune fuite
  de chemin de fichier ; aucune écriture ni figement à la lecture (D12).
- **Requêtes** : N+1, coût du booléen sur un grand journal, index ; CSV et PDF des rapports inchangés (`csv.rs`,
  `pdf.rs`, `benches/export.rs`).
- **`docs/api-external.md`** contre le code : champs, codes d'erreur, exemples.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
