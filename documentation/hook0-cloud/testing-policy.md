---
title: "Testing policy"
slug: "testing-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:34:19 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 12:34:19 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The testing policy describes rules and principles on how testing is applied in our organization.

The policy applies to all developers, internal and external, and outsourced development.

## Principles

- The definition of done contains a test plan/script
- Regression test should be done after each change or update
- There will be no testing with production data:
  - Test data on local laptops and in Test environment is created by developers using fake data
  - Test data on Acceptance is produced by anonymizing the data from Production (replacing all confidential and personally identifiable information by random values)
