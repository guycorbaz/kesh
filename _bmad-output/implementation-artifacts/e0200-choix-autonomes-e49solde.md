# Release 0.20.0 — choix autonomes de la Story 49-5 (clé `e49s`)

Registre de la fiche `49-5-solde-dette-epic` (Opus 5.5, `bmad-create-story`, 2026-11-24). Chaque choix : contexte, option retenue,
autres options, réversibilité.

## C-e49s-1 — Plafond provisoire de trois issues

- **Contexte** : le kickoff de l'Epic 49 n'a pas fixé le plafond de sa story de solde (`CLAUDE.md` § *Priorités des défauts*).
- **Retenu** : trois issues, plafond que le Project Lead a retenu pour l'Epic 53, sur consigne de l'orchestrateur du 2026-11-24 ;
  écrit en tête de la fiche comme « à confirmer par le Project Lead ».
- **Écartées** : deux issues (rien ne le fonde) ; toutes les petites issues (contraire à la règle d'un plafond).
- **Réversibilité** : totale — les trois lots sont indépendants ; la fiche se réduit ou s'étend depuis *Issues écartées*.

## C-e49s-2 — #1399 (PDF) écartée malgré la priorité donnée au PDF

- **Contexte** : la consigne place en tête ce qui touche le PDF des factures. #1399 est confirmée au code
  (`crates/kesh-qrbill/src/pdf.rs:585`, `:624`, `:697`, `:735`).
- **Retenu** : écartée. Son correctif change le PDF de toute facture non encore figée (contrainte 3 du fichier d'épique : réimpression
  à l'identique, seule dérogation admise pour la 49-4b) et ferait rougir la recette de montée de la release (49-1d).
- **Écartées** : la retenir avec une dérogation (décision qui revient au Project Lead, pas à une story de solde trois jours avant
  la release).
- **Réversibilité** : totale ; à présenter au Project Lead avec #1423 (« Plus tard »), même remède.

## C-e49s-3 — #1499 retenue devant #1465 (deux P3)

- **Contexte** : deux P3 confirmées au code, nées de l'épique, pour une place.
- **Retenu** : #1499 (test de la recette : réservation des ports), petite et sans effet sur la production ; elle rend sûr un gate
  que deux voies parallèles lancent chacune.
- **Écartées** : #1465 — son remède est un choix d'écran que la 49-3 (Epic 52) tranche en ouvrant `heading` ; le faire ici poserait
  un état d'écran (quatre langues, recette visuelle) que la 49-3 défera. Proposée au rattachement à la 49-3.
- **Réversibilité** : totale.
