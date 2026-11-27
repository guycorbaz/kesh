# Prompt — passe ciblée sur les précisions d'interface du 2026-11-26 (55-3b, 55-1a, 55-2)

Worktree `<worktree>/kesh-e55`, branche `epic/55-navigation`. Passe **ciblée**, contexte
frais, modèle Opus, en français. Les trois fiches sont au **plafond de huit passes** : cette passe ne compte pour aucune
d'elles ; ses points vont à leur **T0**. Elle juge le commit qui porte le message « précisions d'interface »
(`git log --oneline -3`, puis `git show <ce commit>`), qui verse quatre précisions de l'utilisateur :

1. **Visionneuse (55-3b)** : pas de vignettes de pages à gauche par défaut pour une facture, comme aujourd'hui pour les
   factures fournisseur ; reprendre au code ce qui donne ce comportement (paramètre du lecteur PDF, `#pagemode=none` ou
   équivalent).
2. **Fermer (55-3b)** : un bouton *Fermer* visible, un clic à côté et Échap, ce que le dialogue de Kesh offre déjà, même
   comportement que les autres dialogues.
3. **Exercice en gras (55-1a, AC-14)** : seul le nom en gras, pas les dates — **Exercice 2025** (01.01.2025 →
   31.12.2025).
4. **Icône PDF du Journal (55-3b)** : un peu plus grande qu'une icône de texte, sans augmenter la hauteur de la ligne,
   monochrome de la couleur du texte (ni rouge ni couleur d'alerte), libellé accessible gardé.

Chaque précision doit être un **critère vérifiable**.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — Prémisses au code**, ligne lue recopiée : `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte`
  (l'`<iframe>` et son `src`) ; `grep -rn "pagemode\|navpanes" frontend/src` ; printpdf 0.7
  (`~/.cargo/registry/src/*/printpdf-0.7.0/src/pdf_document.rs`, `/PageMode`) ; bits-ui 2.16.5
  (`frontend/node_modules/bits-ui/dist/bits/utilities/dismissible-layer/dismissible-layer.svelte`,
  `escape-layer/escape-layer.svelte`) et `frontend/src/lib/components/ui/dialog/` ; `frontend/src/lib/features/reports/ReportSelector.svelte`
  et `frontend/src/lib/components/ui/select/` ; `grep -rn selectOption frontend/tests/e2e`.
- **A2 — Fidélité** : chaque précision versée exactement, ni plus ni moins ; la maquette (hors dépôt,
  `$GATE_LOGS/s55-maquettes/mocks-src.html`, images `mocks/m2.png`, `m5a.png` à `m5d.png`) dit-elle
  encore le contraire ?
- **A3 — Constatable** : chaque critère dit comment on le constate (test, mutation, capture) ; un critère invérifiable ou
  vrai par construction est un finding. En particulier : le constat « pas de vignettes » (Vitest + capture) prouve-t-il
  quelque chose ? la mesure de hauteur de ligne à l'E2E est-elle robuste ?
- **A4 — Effets de bord** : le passage de l'AC-14 au composant `ui/select` (E2E, Vitest existants de `ReportSelector`, de
  la page des Rapports, accessibilité, valeur numérique de `selectedFiscalYearId`, la 55-2 et R-5 de la 55-1a) ; le clic à
  côté pendant une lecture ; le focus rendu.
- **A5 — Règles du projet** : i18n, captures, DRY, aucun mot-clé de fermeture, Change Logs datés.
Sévérités : CRITICAL / HIGH / MEDIUM / LOW.

## Interdits

Lecture seule. N'écris aucun fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `node`, `make`, `docker`,
`mariadb`, aucun `scripts/*`, **aucun `gh`**, aucun `git` qui modifie, aucun port ni service. `grep`, `git show`,
`git log`, `sed -n`, lecture d'images admis.

## Rapport

Un seul fichier : `$GATE_LOGS/s55-decisions-2/interface-2026-11-26-validation.md` — verdict chiffré
**par fiche** (55-3b, 55-1a, 55-2), findings (identifiant `UI-n`, fiche, sévérité, preuve, correctif), axes exercés et non
exercés. Puis le même contenu en réponse.
