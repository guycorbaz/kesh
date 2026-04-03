# Change Requests — Kesh

## CR-001 : Réconciliation bancaire

Pouvoir réconcilier les écritures comptables avec les importations de fichiers bancaires (CAMT, MT940, CSV). Rapprochement automatique et/ou manuel entre les transactions importées et les écritures saisies.

## CR-002 : Dashboard configurable

À la connexion, afficher un tableau de bord configurable présentant les valeurs clefs de la comptabilité (soldes des comptes principaux, trésorerie, résultat courant, factures en attente, etc.). L'utilisateur peut choisir quels indicateurs afficher et leur disposition.

## CR-003 : Gestion de stocks simplifiée

Gestion de stocks basique intégrée : suivi des articles, entrées/sorties, valorisation du stock. Adaptée aux besoins d'un indépendant ou d'une petite structure, sans la complexité d'un ERP.

## CR-004 : Calcul d'amortissements (post-MVP)

Gestion des amortissements annuels par investissement. L'utilisateur enregistre un actif (ex : véhicule d'entreprise acheté le 03.03.2026), choisit un type d'amortissement pré-configuré, et Kesh calcule automatiquement les amortissements à déduire et la valeur résiduelle pour chaque exercice.

**Fonctionnalités :**
- Types d'amortissement pré-configurés selon les taux AFC (véhicules, mobilier, informatique, machines, immobilier, etc.)
- Méthode linéaire (montant constant sur la durée) et dégressive (pourcentage sur la valeur résiduelle)
- Calcul au prorata temporis (achat en cours d'année → amortissement proportionnel)
- Tableau d'amortissement par actif : valeur d'acquisition, amortissements cumulés, valeur résiduelle par exercice
- Génération automatique des écritures d'amortissement en fin d'exercice
- Possibilité de créer des types d'amortissement personnalisés (taux et durée libres)
