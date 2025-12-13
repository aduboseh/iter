// SCG MCP Integration Test Suite v2.0
// Production-Ready Boundary Sanitization Tests
//
// This suite validates MCP boundary sanitization under real request/response flows,
// enforcing SCG's doctrine: no DAG topology, ESV internals, energy matrices, or
// lineage hash chains may leak through any endpoint.
//
// Test Categories:
// - Boundary Tests: Core sanitization validation for all endpoints
// - Tool Endpoint Tests: Functional tests for MCP tools
// - Error Handling Tests: Error response sanitization

mod integration;

// Re-export for convenience in tests
pub use integration::common::*;
