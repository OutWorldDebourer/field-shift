# 5. Global Offers

Read on 2026-11-14 from the logged-in offers page: <https://www.spaceappschallenge.org/2026/global-offers/>

## How to claim

Log in at spaceappschallenge.org, click the avatar, choose **Offers**, open the offer and press **Claim Offer**. Each button opens the provider's own site; nothing is activated inside Space Apps. All offers are optional. Questions go to the support contact on each offer or to the Global Offers category of Space Apps Connect: <https://github.com/nasa/spaceapps/discussions/categories/global-offers>

## The offers at a glance

| Offer | What you get | Needs a card | Claim link |
|-------|--------------|--------------|------------|
| GoDaddy Registry: domain and hosting | One free non-premium domain, one year of website hosting and an SSL certificate for that year | Not stated | <https://www.spaceapps.earth/> |
| Google Cloud | The standard free trial: 300 USD of credit for 90 days, no automatic charge afterwards | Yes, for identity check | <https://cloud.google.com/free/docs/free-cloud-features#free-trial> |
| Microsoft Azure | The standard free account for new customers: 200 USD of credit for 30 days plus the always-free and 12-month-free services | Yes, non-prepaid, plus a phone number | <https://azure.microsoft.com/en-us/free/> |
| Microsoft Planetary Computer | Public data catalog and STAC API, no account required; some requests need a free token | No | <https://planetarycomputer.microsoft.com/> |
| GitHub Copilot | The Free plan: up to 2,000 completions per month, limited chat and agent use | No | <https://github.com/features/copilot> |
| Microsoft Copilot | Free assistant at copilot.com | No | <https://www.microsoft.com/en-us/microsoft-copilot> |
| Microsoft Teams | Teams Free: 100 participants, 60-minute meetings, 5 GB per user | No | <https://www.microsoft.com/en-us/microsoft-teams/teams-free> |
| Meteomatics Weather API | 1,000 queries per day on all API features, **only for the duration of the hackathon** | No | <https://www.meteomatics.com/en/meteomatics-and-nasa-space-apps-challenge/> |
| Miro Education | Free for two years for students | No | <https://miro.com/education-whiteboard/> |
| Miro Business | 14-day trial; cancel before it ends to avoid charges | Yes | <https://miro.com/pricing/> |
| Miro templates | Hackathon and Project Coach boards | No | <https://miro.com/templates/profile/nasa-space-apps-challenge/> |

## GoDaddy domain, step by step

This is the only offer with its own claim flow. The page describes four steps:

1. Open <https://www.spaceapps.earth/> and search for the domain name. Only non-premium domains are free.
2. Fill in contact details with a valid email address.
3. Read the email and enter the "launch code" it contains.
4. Finish the free registration with **Porkbun**, GoDaddy's registration partner.

Support: support@porkbun.com. The page does not say which domain endings are included; the claim site itself is on `.earth`, so that is the likely ending (not confirmed).

Completing the flow creates a Porkbun account and enters personal contact data, so the team owner does it personally.

## What this means for the project

- **Google Cloud and Azure are the ordinary public free trials.** They are for new customers only: an account that already used its trial gets no new credit. Both ask for a card.
- **Planetary Computer needs no claim at all.** Its STAC API is the fastest listed route to Landsat, Sentinel-2 and MODIS without creating anything.
- **Meteomatics stops when the hackathon ends**, while judging runs until January 2027. A public demo that depends on it would break for the judges. It is also proprietary data, not NASA data. Use it at most for a one-off comparison, with the result saved.
- **A free domain gives the project a clean public URL** for the "Link to Final Project" field. A new domain should get a FortiGuard classification request: <https://www.fortiguard.com/faq/wfratingsubmit>
- The final project link must open without login, and every non-NASA service used must appear in the references with its license.

## Still open

- [ ] Domain endings and renewal price after the free year.
- [ ] Whether the hosting is required or the domain can point elsewhere by DNS.
- [ ] Whether the team owner's Google and Microsoft accounts are still eligible for the new-customer credits.

## Sources

- Global Offers page (login required): <https://www.spaceappschallenge.org/2026/global-offers/>
- GO Team kickoff message, saved in `Docs/Correo.txt`
- Space Apps Connect discussion 2035: <https://github.com/nasa/spaceapps/discussions/2035>
- Project Submission Guide: <https://www.spaceappschallenge.org/resources/project-submission-guide/>
