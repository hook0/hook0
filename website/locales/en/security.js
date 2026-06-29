// Per-page strings for security (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  "pageTitle": "Hook0 - Security and Compliance",
  "pageDescription": "Learn about Hook0 security practices: GDPR compliance, TLS encryption, cryptographic signatures, and enterprise-grade data protection. Hosted in Europe.",
  "pageModified": "2026-06-29",
  "hero": {
    "eyebrow": "Trust & Safety",
    "h1": "Security & Compliance",
    "subtitle": "We take security seriously. Learn about our comprehensive approach to protecting your data."
  },
  "sections": [
    {
      "h2": "GDPR - Compliance and Certification",
      "cards": [
        {
          "bodyHtml": "If you are dealing with any European Union data through a vendor (like Hook0), then you need a contractual agreement in place with each vendor so the EU knows you're only doing business with companies that fully comply with the General Data Protection Regulation (GDPR)."
        }
      ]
    },
    {
      "h2": "Data Processing Addendum",
      "cards": [
        {
          "bodyHtml": "A data processing agreement (DPA) - also known as a data processing addendum - is a contract between data controllers and data processors or data processors and subprocessors.\n                        <a href=\"./data-processing-addendum\" class=\"text-green-400 hover:text-green-300 transition-colors ml-1\">Learn more</a>."
        }
      ]
    },
    {
      "h2": "Subprocessors",
      "cards": [
        {
          "bodyHtml": "Under the GDPR, a sub-processor is any business or contractor customer data may pass through as a side effect of using Hook0's service.\n                        <a href=\"./gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors ml-1\">Learn more</a>."
        }
      ]
    },
    {
      "h2": "PCI DSS",
      "cards": [
        {
          "bodyHtml": "Hook0's payment and card information is handled by <a href=\"https://stripe.com/docs/security\" class=\"text-green-400 hover:text-green-300 transition-colors\">Stripe</a>, which has been audited by an independent PCI Qualified Security Assessor and is certified as a PCI Level 1 Service Provider, the most stringent level of certification available in the payments industry. Hook0 does not typically receive credit card data, making it compliant with Payment Card Industry Data Security Standards (PCI DSS) in most situations."
        }
      ]
    },
    {
      "h2": "Vulnerability Disclosure",
      "cards": [
        {
          "paragraphs": [
            "If you would like to report a vulnerability or have any security concerns with a Hook0 product, please contact <a href=\"mailto:security@hook0.com\" class=\"text-green-400 hover:text-green-300 transition-colors\">security@hook0.com</a>.",
            "Include a proof of concept, a list of tools used (including versions), and the output of the tools. We take all disclosures very seriously. Once disclosures are received, we rapidly verify each vulnerability before taking the necessary steps to fix it. Once verified, we periodically send status updates as problems are fixed.",
            "If you would like to encrypt sensitive information that you send us, our PGP key can be <a href=\"https://keybase.io/fgribreau\" class=\"text-green-400 hover:text-green-300 transition-colors\">found on Keybase</a>.",
            "We also have an open bug bounty for critical vulnerabilities report regarding Hook0 API (https://app.hook0.com/api/v1/)."
          ]
        }
      ]
    },
    {
      "h2": "Infrastructure and Network Security",
      "cards": [
        {
          "h3": "Physical Access Control",
          "bodyHtml": "Hook0 is hosted on <a href=\"https://www.clever-cloud.com/\" class=\"text-green-400 hover:text-green-300 transition-colors\">Clever Cloud Platform</a>. Clever Cloud data centers feature a layered security model, including extensive safeguards such as:",
          "bullets": [
            "Custom-designed electronic access cards",
            "Alarms and perimeter fencing",
            "Vehicle access barriers and metal detectors",
            "Biometric authentication"
          ],
          "footHtml": "Hook0 employees do not have physical access to Clever Cloud data centers, servers, network equipment, or storage."
        },
        {
          "h3": "Logical Access Control",
          "bodyHtml": "Hook0 is the assigned administrator of its infrastructure on Clever Cloud, and only designated authorized Hook0 operations team members to have access to configure the infrastructure on an as-needed basis behind a two-factor authenticated virtual private network. Specific private keys are required for individual servers, and keys are stored in a secure and encrypted location."
        },
        {
          "h3": "Third-Party Audit",
          "bodyHtml": "Clever Cloud undergoes various third-party independent audits regularly and can provide verification of compliance controls for its data centers, infrastructure, and operations. This includes, but is not limited, to SSAE 16-compliant SOC 2 certification and ISO 27001 certification."
        }
      ]
    },
    {
      "h2": "Business Continuity and Disaster Recovery",
      "cards": [
        {
          "h3": "High Availability",
          "bodyHtml": "Every part of the Hook0 service uses properly-provisioned, redundant servers (e.g., multiple load balancers, web servers, replica databases) in the case of failure. As part of regular maintenance, servers are taken out of operation without impacting availability."
        },
        {
          "h3": "Business Continuity",
          "bodyHtml": "Hook0 keeps hourly encrypted backups of data in multiple regions on Clever Cloud. While never expected, in the case of production data loss (i.e., primary data stores lost), we will restore organizational data from these backups."
        },
        {
          "h3": "Disaster Recovery",
          "bodyHtml": "In the event of a region-wide outage, Hook0 will bring up a duplicate environment in a different Clever Cloud region. The Hook0 operations team has extensive experience performing full region migrations."
        }
      ]
    },
    {
      "h2": "Corporate Security",
      "cards": [
        {
          "h3": "Malware Protection",
          "bodyHtml": "At Hook0, we believe that good security practices start with our own team, so we go out of our way to protect against internal threats and local vulnerabilities."
        },
        {
          "h3": "Risk Management",
          "paragraphs": [
            "Hook0 follows the risk management procedures outlined in <a href=\"http://csrc.nist.gov/publications/PubsSPs.html\" class=\"text-green-400 hover:text-green-300 transition-colors\">NIST SP 800-30</a>, which include nine steps for risk assessment and seven steps for risk mitigation.",
            "All Hook0 product changes must go through code review, CI, and build pipeline to reach production servers. Only designated employees on Hook0's operations team have secure shell (SSH) access to production servers.",
            "Hook0 performs risk assessments throughout the product lifecycle per the standards outlined in <a href=\"https://www.law.cornell.edu/cfr/text/45/164.308\" class=\"text-green-400 hover:text-green-300 transition-colors\">HIPAA Security Rule, 45 CFR 164.308</a>."
          ]
        },
        {
          "h3": "Security Policies & Training",
          "bodyHtml": "Hook0 maintains an internal wiki of security policies, which is updated on an ongoing basis and reviewed annually for gaps. All new employees receive onboarding and systems training, including security policies review."
        },
        {
          "h3": "Disclosure Policy",
          "paragraphs": [
            "Hook0 follows the incident handling and response process recommended by <a href=\"https://www.sans.org/reading-room/whitepapers/incident/incident-handlers-handbook-33901\" class=\"text-green-400 hover:text-green-300 transition-colors\">SANS</a>, which includes identifying, containing, eradicating, recovering from, communicating, and documenting security events.",
            "Hook0 maintains a live report of operational uptime and issues on our <a href=\"https://status.hook0.com/\" class=\"text-green-400 hover:text-green-300 transition-colors\">status page</a>. Any known incidents are reported there, as well as on our <a href=\"https://twitter.com/hook0_\" class=\"text-green-400 hover:text-green-300 transition-colors\">Twitter feed</a>."
          ]
        }
      ]
    }
  ]
};
