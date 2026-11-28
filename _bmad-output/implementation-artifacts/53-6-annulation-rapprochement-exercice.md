# Story 53-6 : découpée le 2026-11-27 en 53-6a et 53-6b (#1674)

Status: split

Cette fiche est une **page de renvoi**. La story 53-6 (annulation d'un rapprochement contre-passée à la date du jour, P1 #1674) a été
**découpée le 2026-11-27 sur décision du Project Lead**. **Raison** : la validation P2 a rendu trois MEDIUM sur trois nés d'un remède de
la P1 — le signal de recyclage de la règle de splitting préventif (CLAUDE.md) ; les défauts portaient sur le texte de la fiche, non sur la
conception du correctif.

- **53-6a — le correctif** : `53-6a-annulation-rapprochement-date-origine.md` (statut `ready-for-dev`, sous réserve de sa validation).
- **53-6b — la détection des livres déjà touchés** : `53-6b-detection-contre-passations-autre-exercice.md` (statut `backlog`, dépend de la 53-6a).

Les questions Q1 à Q6 sont tranchées le 2026-11-27 et écrites dans la 53-6a. La fiche d'origine, ses validations P1 et P2 et leurs prompts
restent dans l'historique git (`git log -p` de ce fichier ; commit `c692c3406`) et dans `kesh-gate-logs/e53/53-6-validation-p1.md` et `…-p2.md`.
