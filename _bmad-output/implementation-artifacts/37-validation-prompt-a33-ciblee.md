# Prompt de validation ciblée — A33 (frais de conversion), fiche 37-3b

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), contexte frais. Worktree `/home/beat/limu/compta/kesh-wt37`
(branche `epic/37-multi-devises`) : n'écris nulle part ailleurs.

## Périmètre

- La remédiation à relire : `git diff c6274a58 3c98a3dc` (lecture seule) — arbitrage A33 au fichier d'epic et son application à
  `37-3b-ecritures-en-devise-ecrans.md` (AC-2 bis, assistant *Conversion*, guide d'API, FR-3, FR-8, tests).
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche,
  avec A31, A32 et 37-3a (AC-4, 2 bis : `FX_MULTIPLE_CURRENCIES`), comptablement juste (frais au débit d'un compte de
  charge de la même devise, différence calculée après, écriture équilibrée, sens gain/perte) ? Le changement de code de
  refus (deux devises → `FX_MULTIPLE_CURRENCIES` au lieu de `ENTRY_UNBALANCED`) est-il cohérent avec l'ordre des refus
  et les tests existants ? Un frais en CHF (compte de charge sans devise) sur l'écriture complétée : quel sort ?
- Hors périmètre : le reste de la fiche.

## Interdits

N'écris que dans la fiche 37-3b et la ligne A33 / récapitulatif de l'epic, par **Edit** seulement. Aucun script, aucun
`cargo`/`npm`, aucun `git` en écriture.

## Rapport (français)

Findings `37-3b-A33-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-15 — validation ciblée A33 (<modèle>) : <décompte> ». Message final bref.
