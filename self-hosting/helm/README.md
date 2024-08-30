# Hook0 Helm Chart

## What is Hook0?

Hook0 is an advanced webhook processing and event management service designed for scalable and reliable integrations.

This Helm chart simplifies the deployment of Hook0 on Kubernetes clusters, providing an easy and efficient way to manage
and process webhooks across various applications.

## Prerequisites

Before deploying Hook0, ensure you have the following prerequisites met:

- **Kubernetes Cluster**: A running Kubernetes cluster (version 1.19 or later).
- **kubectl**: Kubernetes command-line tool installed and configured to communicate with your cluster.
- **Helm**: Helm package manager installed (version 3.x).
- **Persistent Storage**: Access to a storage class for provisioning PersistentVolumeClaims.
- **Ingress Controller**: (Optional but recommended) An ingress controller like NGINX for external access.

## Installation

### 1. Clone the Repository

Clone the Hook0 Helm chart repository to your local machine:

```bash
git clone https://github.com/hook0/hook0.git
cd hook0/self-hosting/helm
```

### 2. Customize Configuration (Optional)

Modify the `values.yaml` file to tailor the deployment to your needs. Key configurations include:

```yaml
api:
  replicaCount: 2
  image:
    repository: your-docker-repo/hook0-api
    tag:        latest
  env:
    APP_URL:              "https://api.yourdomain.com"
    DATABASE_URL:         "postgres://user:password@postgres:5432/hook0"
    EMAIL_SENDER_ADDRESS: "noreply@yourdomain.com"
    SMTP_CONNECTION_URL:  "smtp://smtp.yourdomain.com:587"
frontend:
  replicaCount: 2
  image:
    repository: your-docker-repo/hook0-frontend
    tag:        latest
  service:
    type: ClusterIP
    port: 80
postgres:
  persistence:
    enabled:      true
    storageClass: "standard"
    size:         10Gi
mailpit:
  enabled: true
  persistence:
    enabled:      true
    storageClass: "standard"
    size:         1Gi
ingress:
  enabled: true
  hosts:
    - host: "hook0.yourdomain.com"
      paths:
        - path:     "/"
          pathType: Prefix
```

**Note**: Replace `your-docker-repo` and `yourdomain.com` with your actual Docker repository and domain names.

### 3. Deploy Hook0 using Helm

Install the Hook0 Helm chart with your customized configurations:

```bash
helm install hook0 . -n hook0 --create-namespace
```

This command deploys all Hook0 components into the `hook0` namespace.

### 4. Verify the Deployment

Check the status of the deployed resources:

```bash
kubectl get all -n hook0
```

Ensure all pods are in the `Running` state and services are correctly configured.

### 5. Accessing Hook0 Services

#### Via Ingress

If you've enabled and configured Ingress:

- **API**: Access via `https://api.yourdomain.com`
- **Frontend**: Access via `https://hook0.yourdomain.com`

#### Without Ingress

Retrieve the service details and access them using the cluster IPs or set up port forwarding:

```bash
kubectl port-forward svc/frontend 8080:80 -n hook0
```

Now access the frontend at `http://localhost:8080`.

## Configuration

### Environment Variables

Sensitive information such as database credentials should be managed securely using Kubernetes Secrets. Update the
`values.yaml` to reference these secrets:

```yaml
api:
  env:
    DATABASE_URL:
      valueFrom:
        secretKeyRef:
          name: hook0-database-secret
          key:  database_url
```

Create the secret using:

```bash
kubectl create secret generic hook0-database-secret   --from-literal=database_url='postgres://user:password@postgres:5432/hook0'   -n hook0
```

### Scaling

Adjust the number of replicas for API and frontend services based on your load requirements:

```yaml
api:
  replicaCount: 3
frontend:
  replicaCount: 3
```

Apply the changes:

```bash
helm upgrade hook0 . -n hook0
```

## Monitoring and Logging

Integrate monitoring and logging solutions to keep track of Hook0 performance and health:

- **Monitoring**: Use Prometheus and Grafana to monitor metrics.
- **Logging**: Use Elasticsearch, Fluentd, and Kibana (EFK stack) for centralized logging.

Example integration can be added to the `values.yaml`:

```yaml
monitoring:
  enabled: true
logging:
  enabled: true
```

Ensure your cluster has these tools installed and configured appropriately.

## Security Considerations

- **TLS/SSL**: Configure TLS certificates for secure communication, especially if exposing services over the internet.
  Use cert-manager for automatic certificate management.
- **Network Policies**: Implement Kubernetes NetworkPolicies to restrict traffic between pods and services.
- **Resource Quotas**: Define resource quotas and limits to prevent resource exhaustion.
- **Regular Updates**: Keep all images and dependencies up-to-date to incorporate security patches.

## Troubleshooting

### Common Issues and Solutions

- **Pods Not Starting**: Check logs using `kubectl logs` and describe the pod for events using `kubectl describe pod`.
- **Persistent Volume Claims Pending**: Ensure the specified storage class exists and has sufficient resources.
- **Services Not Accessible**: Verify service configurations and ensure Ingress rules are correctly set up.
- **Image Pull Errors**: Confirm that the Docker images are accessible and credentials are correctly provided if using
  private repositories.

### Getting Support

- **Documentation**: Refer to the [Hook0 Official Documentation](https://www.hook0.com/docs) for detailed guides.
- **Community Support**: Join the Hook0 community on [Slack](https://hook0.slack.com)
  or [GitHub Discussions](https://github.com/hook0/hook0/discussions).
- **Report Issues**: Open issues on the [GitHub repository](https://github.com/hook0/hook0/issues) with detailed
  descriptions.

## Uninstallation

To remove Hook0 from your cluster:

```bash
helm uninstall hook0 -n hook0
```

This command deletes all Kubernetes resources associated with the Hook0 release. To delete the namespace entirely:

```bash
kubectl delete namespace hook0
```

**Caution**: Ensure you have backed up any persistent data before uninstallation.

## Contributing

Contributions are welcome! Please read
the [contributing guidelines](https://github.com/hook0/hook0/blob/main/CONTRIBUTING.md) before submitting pull requests.

## License

This project is licensed under the [MIT License](https://github.com/hook0/hook0/blob/main/LICENSE).

## Acknowledgements

Thanks to all the contributors and the open-source community for making this project possible.
