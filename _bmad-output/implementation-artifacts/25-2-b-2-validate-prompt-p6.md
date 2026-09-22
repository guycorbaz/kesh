# Prompt — passe 6 de `bmad-create-story validate`, Story 25-2-b-2 — passe CIBLÉE

*Versionné le 2026-09-21. Une lentille en contexte frais (Opus) — les passes 1, 3 et 5 étaient sur
Sonnet, les passes 2 et 4 sur Opus, comme l'auteur des patches.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api` (elle porte `main`
au `951cbce2`, release `v0.12.0` comprise).

## L'objet, et pourquoi cette passe existe

```sh
git show <le commit de tête> -- _bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md
```

⛔ **La passe 5 a rendu un CRITICAL de procédure** : le Change Log de la passe 4 déclarait **onze
corrections** et le commit n'en portait **qu'une** — un script de remédiation avait échoué avant
d'écrire le fichier. Les treize corrections ont été **réappliquées**, et c'est ce patch-là que tu
relis.

⚠️ **Tu ne crois donc AUCUNE déclaration de compte rendu.** Le Change Log dit ce qui devrait être
fait ; **le fichier dit ce qui est fait**, et c'est le fichier qui fait foi. **Commence par
confronter la dernière ligne du Change Log au corps des critères, point par point** — c'est
exactement ce que la passe 5 a fait, et ce qu'elle a trouvé.

## Les axes, et tu déclareras lesquels tu as exercés

1. ⛔ **Chaque correction déclarée est-elle DANS le corps de la fiche ?** Les treize : le test neuf
   retiré de l'AC 1 ; « NEUF AUTRES sites » et sa ventilation (+ les deux déjà nommés = onze
   ancrages) ; `invoices.rs:1232-1235`, `:1236-1282`, `:1331`, `:1339-1349` ; le renommage des deux
   tests retargetés ; `delete_validated_in_locked_period_…` **réécrit** et sorti de la cellule
   « supprimés » ; la garde `isAdmin` restreinte à `:626-629` ; le retrait d'écran `:919-943` **et**
   `:945` plus l'import `:3` ; « ne devient pas inconditionnel » ; le **second chemin de
   suppression** (écran de liste, `:423-429` et `:459-491`) dans l'AC 5 **et** la garde de rôle de
   l'AC 4 ; l'AC 10 qui **crée** `## [X.Y.Z] — Non publié` et **interdit** de toucher au 123 de
   `[0.12.0]` ; la référence à l'écran de liste ; la tâche T1.
2. **Chaque correction est-elle JUSTE ?** Vérifie les lignes sur l'arbre de la branche — c'est là que
   les passes 2, 3 et 4 ont chacune introduit une citation fausse.
3. ⛔ **Le format de la section du `CHANGELOG`.** `scripts/prepare-release.sh` contrôle un motif
   exact en pré-vol : lis-le, et dis si ce que l'AC 10 prescrit le satisfait — un faux pas ici
   **bloquerait la prochaine release**.
4. **Les corrections créent-elles leur propre défaut ?** Contradictions internes, référence croisée
   cassée entre les deux filles, total incohérent avec sa ventilation.
5. **Ce que personne n'a encore énuméré.** Les chemins — pas les écrans — qui appelleront
   `unvalidate` ou `deleteInvoice` ; les sites qui lisent `invoice_number` en supposant un statut ;
   tout registre ou compteur que cette story fera bouger et que la fiche ne nomme pas.

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la preuve**, le correctif. Pour chacun, dis s'il
  est de nature **découpage**, **conception**, **dérive extérieure** ou **procédure**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** ⚠️ **Un « 0 finding »
  non adossé à cette liste ne clôt rien.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh` (**le lire, jamais le lancer** : il bumpe les dix crates),
`scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`add`/`stash`/`reset`/`merge`, `sqlx migrate`,
`cargo test`/`cargo nextest`. Lecture, `grep`, `sed -n`, `git show`/`diff`/`log`, `pdftotext` et
`cargo check` sont autorisés.
