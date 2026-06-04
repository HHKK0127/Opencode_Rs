use serde::{Deserialize, Serialize};
use tracing::info;

/// Query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub query_id: String,
    pub original_query: String,
    pub optimized_query: String,
    pub estimated_cost: f64,
    pub optimization_applied: Vec<String>,
}

impl QueryPlan {
    pub fn new(query_id: String, query: String) -> Self {
        Self {
            query_id,
            original_query: query.clone(),
            optimized_query: query,
            estimated_cost: 100.0,
            optimization_applied: Vec::new(),
        }
    }

    pub fn add_optimization(&mut self, optimization: String) {
        self.optimization_applied.push(optimization);
    }

    pub fn estimated_improvement(&self) -> f64 {
        // Estimate improvement based on optimizations
        let base_cost = 100.0;
        let reduction = self.optimization_applied.len() as f64 * 10.0;
        (base_cost - reduction.min(base_cost)) / base_cost * 100.0
    }
}

/// Query optimizer
pub struct QueryOptimizer {
    optimization_rules: Vec<String>,
}

impl QueryOptimizer {
    pub fn new() -> Self {
        Self {
            optimization_rules: vec![
                "Add index on frequently filtered columns".to_string(),
                "Use index for sorting operations".to_string(),
                "Reorder join conditions".to_string(),
                "Cache frequently accessed data".to_string(),
                "Use pagination for large result sets".to_string(),
                "Batch multiple queries".to_string(),
            ],
        }
    }

    /// Optimize query
    pub fn optimize(&self, query: &str) -> QueryPlan {
        let query_id = uuid::Uuid::new_v4().to_string();
        let mut plan = QueryPlan::new(query_id, query.to_string());

        // Apply optimizations based on query patterns
        if query.to_uppercase().contains("ORDER BY") {
            plan.add_optimization("Use index for sorting".to_string());
        }

        if query.to_uppercase().contains("WHERE") {
            plan.add_optimization("Add filter index".to_string());
        }

        if query.to_uppercase().contains("JOIN") {
            plan.add_optimization("Optimize join order".to_string());
        }

        if query.to_uppercase().contains("LIMIT") && query.to_uppercase().contains("OFFSET") {
            plan.add_optimization("Use pagination".to_string());
        }

        // Update estimated cost based on optimizations
        plan.estimated_cost = 100.0 - (plan.optimization_applied.len() as f64 * 15.0);

        info!("Query optimized: {} optimizations applied", plan.optimization_applied.len());
        plan
    }

    /// Analyze query complexity
    pub fn analyze_complexity(&self, query: &str) -> String {
        let mut complexity = "Simple".to_string();

        let join_count = query.to_uppercase().matches("JOIN").count();
        let where_count = query.to_uppercase().matches("WHERE").count();

        if join_count > 3 || where_count > 5 {
            complexity = "Complex".to_string();
        } else if join_count > 1 || where_count > 2 {
            complexity = "Moderate".to_string();
        }

        complexity
    }

    pub fn get_rules(&self) -> Vec<String> {
        self.optimization_rules.clone()
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_plan_creation() {
        let plan = QueryPlan::new(
            "q1".to_string(),
            "SELECT * FROM files WHERE user_id = 123".to_string(),
        );

        assert_eq!(plan.original_query, "SELECT * FROM files WHERE user_id = 123");
        assert_eq!(plan.optimization_applied.len(), 0);
    }

    #[test]
    fn test_query_optimization() {
        let optimizer = QueryOptimizer::new();
        let plan = optimizer.optimize(
            "SELECT * FROM files WHERE user_id = 123 ORDER BY created_at DESC LIMIT 20 OFFSET 0",
        );

        assert!(!plan.optimization_applied.is_empty());
        assert!(plan.estimated_cost < 100.0);
    }

    #[test]
    fn test_query_complexity_analysis() {
        let optimizer = QueryOptimizer::new();

        let simple = optimizer.analyze_complexity("SELECT * FROM files");
        assert_eq!(simple, "Simple");

        // Moderate: 2 JOINs or 3+ WHERE clauses
        let moderate = optimizer.analyze_complexity(
            "SELECT f.*, u.name FROM files f JOIN users u ON f.user_id = u.id \
             JOIN posts p ON u.id = p.user_id WHERE f.size > 1000",
        );
        assert_eq!(moderate, "Moderate");
    }

    #[test]
    fn test_estimated_improvement() {
        let mut plan = QueryPlan::new("q1".to_string(), "SELECT * FROM files".to_string());
        plan.add_optimization("Index".to_string());
        plan.add_optimization("Cache".to_string());

        let improvement = plan.estimated_improvement();
        assert!(improvement > 0.0);
    }
}
