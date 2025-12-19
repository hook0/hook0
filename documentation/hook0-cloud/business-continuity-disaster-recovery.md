---
title: "Business Continuity and Disaster Recovery"
slug: "business-continuity-and-disaster-recovery"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Tue Jul 25 2023 12:43:04 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Tue Jul 25 2023 13:35:44 GMT+0000 (Coordinated Universal Time)"
---
This Business Continuity and Disaster Recovery (BCDR) plan outlines the steps that Hook0 will take to resume critical business operations in the event of a disaster. The document includes procedures for identifying, preventing, and mitigating risks; recovering from disruptions; and restoring normal operations. Since Hook0 is mainly hosted on CleverCloud some steps are directly related to CleverCloud own Disaster Recovery and Business Continuity plan.

## Scope

This BCDR applies to all Hook0 employees and contractors. It covers all aspects of Hook0's business, including information technology, facilities, and operations.

## Risks

Hook0 faces a number of risks that could disrupt its business operations. These risks include:

1. **Natural disasters** (earthquakes, floods, fires)
2. **Cyber-attacks**
3. **Data corruption or loss**
4. **Infrastructure failures**
5. **Human errors**

## Risk Mitigation

Hook0 has implemented a number of controls to mitigate the risks identified above. These controls include:

1. **Data backups**: Regular and encrypted backups with off-site storage from CleverCloud managed PostgreSQL database.
2. **Infrastructure redundancy**: Multiple server locations and cloud-based failover provided by CleverCloud PaaS.
3. **Regular security audits through Bug Bounties**: To identify and fix vulnerabilities.
4. **Employee training**: To reduce human errors and phishing risks.

## Response Strategy

1. **Incident Identification**: Detect and classify the severity of the incident.
2. **Initial Communication**: Notify stakeholders and form a response team.
3. **Incident Isolation**: Prevent the incident from causing more damage.
4. **Service Restoration**: Restore critical services first.
   1. Restore critical systems and data from CleverCloud's backups.
   2. Relocate operations to an alternate site, if necessary, as determined by CleverCloud.
5. **Recovery**: Bring all systems back to normal.

## Restoration Procedures

Once Hook0 has restored critical systems and data, it will begin the process of restoring normal operations. This process will include:

- Restoring employees to their workstations.
- Restoring business processes.
- Restoring customer relationships.

## Recovery Time Objectives (RTO) & Recovery Point Objectives (RPO)

Every customer has the default RTO and RPO:

| Service           | RTO     | RPO   |
| ----------------- | ------- | ----- |
| IT Infrastructure | 3 hours | N/A   |
| Customer Support  | 2 hours | N/A   |
| Data Restoration  | 1 hour  | 1 day |

For customers with dedicated custom subscription plans please look at your contract if you asked for lower RPO and RTO.

## Communication Plan

- **Internal Communication**: Use internal chat tools and backup communication tools (e.g., cell phones).
- **External Communication**: Update customers via the website, emails, and social media channels.

## Testing and Maintenance

Hook0 will test and maintain its BCDR on a regular basis to ensure that it is effective. Testing will include:

- Simulation exercises conducted by CleverCloud
- Tabletop exercises conducted by Hook0

Maintenance will include:

- Reviewing and updating the plan
- Training employees on the plan

## Conclusion

This BCDR provides Hook0 with a roadmap for resuming critical business operations in the event of a disaster. By following the procedures outlined in this BCDR, Hook0 can minimize the impact of a disaster and ensure that it can continue to serve its customers.

### Additional Information

#### CleverCloud BCDR

CleverCloud has a comprehensive disaster recovery plan that is designed to ensure the continuity of service for its customers. The plan includes the following key elements:

- Daily backups of all customer data
- Multiple data centers in different geographic locations
- The ability to rapidly restore services from backup
- A team of experts dedicated to disaster recovery

In the event of a disaster, CleverCloud will activate its disaster recovery plan and work to restore services as quickly as possible. Customers will be kept informed of the status of the recovery effort and will be provided with updates as they become available.

For more information on CleverCloud's disaster recovery plan, please visit the [CleverCloud website](https://www.clever-cloud.com/fr/).

#### CleverCloud PostgreSQL Backup Policy

CleverCloud automatically backs up all PostgreSQL databases on a daily basis. The backups are stored in multiple data centers in different geographic locations. In the event of a disaster, CleverCloud can restore a database from backup in a matter of minutes.

For more information on CleverCloud's PostgreSQL backup policy, please visit the [CleverCloud website](https://www.clever-cloud.com/fr/).
