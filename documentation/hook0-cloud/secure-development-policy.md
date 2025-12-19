---
title: "Secure development policy"
slug: "secure-development-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:45:18 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 12:45:18 GMT+0000 (Coordinated Universal Time)"
---
Alias: Application Security Policy

## Summary

The secure development policy defines rules and principles applied in software development.

The policy applies to all developers, internal and external. 

Secure development practices will be established, implemented, and documented for all applications developed or purchased to include appropriate security controls to prevent unauthorized access or modification of the system or information coded or stored. 

## Principles

### Source control

- All source code is stored in a code repository
- All checked in source code must compile without warnings
- Code should be reviewed by a peer before it can be committed to Acceptance

### Development

- Developing a test plan/script is part of all user stories
- Privacy and security are part of any design
- Developers should be aware of common threats and vulnerabilities (through initiatives as [OWASP](https://www.owasp.org/index.php/Main_Page))

### Libraries and frameworks

The use of libraries and frameworks is encouraged, but

- The versions should be periodically assessed for vulnerabilities
- Use the library or framework as-is, refrain from making changes (this makes updating a lot more complicated, and it may pose licensing problems)
