{{- define "hook0-acquisition.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "hook0-acquisition.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "hook0-acquisition.labels" -}}
app.kubernetes.io/name: {{ include "hook0-acquisition.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "hook0-acquisition.selectorLabels" -}}
app.kubernetes.io/name: {{ include "hook0-acquisition.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
