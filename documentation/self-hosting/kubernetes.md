---
title: Kubernetes
description: Deploy Hook0 on a Kubernetes cluster
---

# Kubernetes

This guide provides instructions for deploying Hook0 on a Kubernetes cluster using a `deployments.yaml` file.

## Prerequisites

- A functioning Kubernetes cluster
- `kubectl` tool access
- Persistent storage capacity for PostgreSQL and Mailpit services

## Core Components

The deployment establishes five main services:

- API endpoint
- Frontend interface
- Mailpit for email management
- PostgreSQL database
- Output Worker for webhook processing

## Deployment Process

### 1. Namespace Creation

First, create a dedicated `hook0` namespace:

```bash
kubectl create namespace hook0
```

### 2. Service Deployment

Apply the complete YAML configuration file:

```bash
kubectl apply -f deployments.yaml -n hook0
```

### 3. Verification

Confirm all pods reach "Running" state:

```bash
kubectl get pods -n hook0
```

Troubleshoot failures with pod descriptions:

```bash
kubectl describe pod <pod-name> -n hook0
```

### 4. Service Access

Retrieve external IPs or NodePorts:

```bash
kubectl get services -n hook0
```

### 5. Storage Management

PVCs automatically handle data persistence, with default 100Mi allocations per service.

### 6. Configuration Customization

Modify environment variables like `DATABASE_URL` and `APP_URL` as needed in the deployment files.

### 7. Troubleshooting

Common issues to address:

- Image availability
- Storage class compatibility
- Network access configuration

## Cleanup

To delete the Hook0 deployment:

```bash
kubectl delete namespace hook0
```

This removes all associated cluster resources.
