# Admin User Stories

## Overview

This document contains user stories for all features of Teradata Performance Monitor (PMON), a real-time monitoring tool that displays system status and activity for Teradata databases.

---

## 1. Configuration Summary

| ID | User Story |
|----|------------|
| US-1.1 | As a DBA, I want to view the current Teradata system configuration at a glance so that I can quickly understand the system topology and verify the setup is correct. |
| US-1.2 | As a system administrator, I want to see the number of nodes, AMPs, and PEs in my system so that I can confirm my hardware resources match expected capacity. |
| US-1.3 | As an operations analyst, I want to access configuration details without running multiple queries so that I can save time during routine system checks. |

---

## 2. Performance Summary and Resource Usage (Physical and Virtual)

| ID | User Story |
|----|------------|
| US-2.1 | As a DBA, I want to monitor CPU usage per VPROC in virtual mode so that I can identify which virtual processors are experiencing high load. |
| US-2.2 | As a DBA, I want to monitor CPU usage per node in physical mode so that I can identify hardware-level bottlenecks. |
| US-2.3 | As a performance engineer, I want to view real-time disk I/O metrics so that I can detect storage bottlenecks affecting query performance. |
| US-2.4 | As a capacity planner, I want to see memory utilization across the system so that I can determine if additional resources are needed. |
| US-2.5 | As an operations analyst, I want to compare physical versus virtual resource consumption so that I can understand the relationship between hardware and logical resource allocation. |
| US-2.6 | As a DBA, I want to identify uneven CPU or I/O usage across VPROCs so that I can investigate potential data skew issues. |

---

## 3. Session and Lock Information

| ID | User Story |
|----|------------|
| US-3.1 | As a DBA, I want to view all active sessions on the system so that I can monitor current user activity. |
| US-3.2 | As a DBA, I want to identify blocked sessions so that I can investigate and resolve contention issues. |
| US-3.3 | As an operations analyst, I want to see which sessions are holding locks so that I can determine the cause of blocking. |
| US-3.4 | As a DBA, I want to view the user associated with each session so that I can contact them if their query is causing problems. |
| US-3.5 | As a support engineer, I want to see session states (idle, active, blocked, responding, parsing, aborting) so that I can quickly assess system health. |
| US-3.6 | As a DBA, I want to identify which sessions are blocking other sessions and why so that I can make informed decisions about intervention. |
| US-3.7 | As an application support analyst, I want to view query states for active sessions so that I can troubleshoot application performance issues. |

---

## 4. Session History

| ID | User Story |
|----|------------|
| US-4.1 | As a DBA, I want to view historical session activity so that I can analyze patterns over time. |
| US-4.2 | As a performance analyst, I want to review past session behavior so that I can identify recurring issues. |
| US-4.3 | As an operations manager, I want to track session trends so that I can plan for peak usage periods. |
| US-4.4 | As a capacity planner, I want to analyze historical session counts so that I can forecast future resource needs. |
| US-4.5 | As a DBA, I want to compare current session activity against historical baselines so that I can detect anomalies. |

---

## 5. Control Functions

| ID | User Story |
|----|------------|
| US-5.1 | As a DBA, I want to abort a runaway session so that I can free up system resources for other users. |
| US-5.2 | As a DBA, I want to abort a specific query without killing the entire session so that the user can continue working. |
| US-5.3 | As a system administrator, I want to release locks held by problematic sessions so that I can unblock other waiting sessions. |
| US-5.4 | As a DBA, I want to change the priority of a session so that I can adjust resource allocation based on business needs. |
| US-5.5 | As an operations analyst, I want to log off idle sessions so that I can reclaim system resources. |
| US-5.6 | As a DBA, I want to abort all sessions for a specific user so that I can quickly respond to a security incident or misbehaving application. |
| US-5.7 | As a support engineer, I want to abort sessions on a specific host so that I can isolate issues related to a particular client machine. |

---

## 6. Graphic Displays of Resource Data

| ID | User Story |
|----|------------|
| US-6.1 | As a DBA, I want to view CPU utilization in a graphical chart so that I can quickly spot trends and anomalies. |
| US-6.2 | As a performance engineer, I want to see I/O activity visualized over time so that I can correlate spikes with specific workloads. |
| US-6.3 | As an operations analyst, I want to view resource metrics in a graphical format so that I can easily present system status to management. |
| US-6.4 | As a DBA, I want to see color-coded warnings when resource thresholds are exceeded so that I can immediately identify problem areas. |
| US-6.5 | As a capacity planner, I want to visualize resource consumption patterns so that I can identify peak usage times. |
| US-6.6 | As a system administrator, I want graphical displays that highlight abnormalities so that I can focus attention on areas requiring action. |

---

## 7. Graphic Displays of Session Data

| ID | User Story |
|----|------------|
| US-7.1 | As a DBA, I want to view session counts in a graphical chart so that I can monitor connection trends. |
| US-7.2 | As an operations analyst, I want to see the distribution of session states (idle, active, blocked) visually so that I can quickly assess system health. |
| US-7.3 | As a performance engineer, I want to visualize session activity over time so that I can identify patterns in user behavior. |
| US-7.4 | As a support engineer, I want to see prolonged idle sessions highlighted so that I can investigate potential connection leaks. |
| US-7.5 | As a DBA, I want to view blocked session trends graphically so that I can identify recurring contention problems. |

---

## 8. Alerting and Threshold Configuration

| ID | User Story |
|----|------------|
| US-8.1 | As a DBA, I want to configure alert thresholds for resource usage so that I am notified when metrics exceed acceptable levels. |
| US-8.2 | As a system administrator, I want to customize color settings for warning conditions so that I can align with my organization's monitoring standards. |
| US-8.3 | As an operations analyst, I want to set the automatic data refresh rate so that I can balance between real-time visibility and system overhead. |
| US-8.4 | As a DBA, I want alerts to use color indicators so that I can immediately recognize warning conditions without reading detailed metrics. |
| US-8.5 | As a support engineer, I want to configure different thresholds for different metrics so that I can prioritize the most critical resources. |

---

## 9. Query Drill-Down and Analysis

| ID | User Story |
|----|------------|
| US-9.1 | As a DBA, I want to drill down from a blocked session to the underlying query so that I can understand what SQL is causing the issue. |
| US-9.2 | As a performance engineer, I want to view the explain plan steps for a running query so that I can identify inefficient execution steps. |
| US-9.3 | As a support analyst, I want to analyze running queries in real-time so that I can find critical steps causing performance problems. |
| US-9.4 | As a DBA, I want to identify queries with heavy AMP skewing so that I can recommend optimization strategies. |
| US-9.5 | As a developer, I want to monitor my query while it runs so that I can observe its behavior and identify tuning opportunities. |

---

## 10. Dynamic Session Monitoring

| ID | User Story |
|----|------------|
| US-10.1 | As a DBA, I want session status to update automatically every six seconds so that I can see near real-time system activity. |
| US-10.2 | As an operations analyst, I want to monitor sessions on all logical host processors so that I have complete visibility across the system. |
| US-10.3 | As a support engineer, I want session information to refresh dynamically so that I don't need to manually reload data during troubleshooting. |
| US-10.4 | As a DBA, I want to adjust the refresh frequency based on system load so that monitoring doesn't impact production performance. |

---

## Summary by Persona

| Persona | User Story Count |
|---------|------------------|
| DBA | 22 |
| Operations Analyst | 8 |
| Performance Engineer | 5 |
| Support Engineer | 6 |
| System Administrator | 4 |
| Capacity Planner | 4 |
| Developer | 1 |
| Operations Manager | 1 |
| Application Support Analyst | 1 |
| Performance Analyst | 1 |
| Support Analyst | 1 |

---

## Document Information

| Field | Value |
|-------|-------|
| Total User Stories | 54 |
| Feature Categories | 10 |
| Created Date | February 2026 |
| Source | Teradata PMON Documentation |
