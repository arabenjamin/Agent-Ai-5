use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use neo4rs::{Graph, Node, Query, Relation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

// Context node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextNodeType {
    Metric,
    SystemState,
    UserInteraction,
    ToolExecution,
    Pattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNode {
    pub node_type: ContextNodeType,
    pub timestamp: DateTime<Utc>,
    pub properties: HashMap<String, serde_json::Value>,
}

// Relationship types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    Followed,
    Caused,
    Related,
    Contains,
    Triggered,
}

lazy_static! {
    static ref NEO4J_CLIENT: Mutex<Option<Graph>> = Mutex::new(None);
}

pub struct Neo4jContext {
    graph: Graph,
}

impl Neo4jContext {
    pub async fn connect(url: String, user: String, password: String) -> Result<Neo4jContext, Box<dyn Error + Send + Sync>> {
        info!("Attempting to connect to Neo4j at {}", url);
        debug!("Establishing Neo4j connection...");
        let uri = url.as_str();
        
        // Try to connect with retries
        let mut retries = 5;
        let mut last_error = None;
        
        while retries > 0 {
            debug!("Attempting connection (retries left: {})", retries);
            match Graph::new(uri, user.as_str(), password.as_str()).await {
                Ok(graph) => {
                    info!("Successfully connected to Neo4j");
                    debug!("Neo4j connection established and verified");
                    
                    // Initialize schema after connecting
                    if let Err(e) = Self::init_schema(&graph).await {
                        error!("Failed to initialize Neo4j schema: {}", e);
                        return Err(e);
                    }
                    
                    return Ok(Neo4jContext { graph });
                }
                Err(e) => {
                    error!("Connection attempt failed: {}", e);
                    last_error = Some(e);
                    retries -= 1;
                    if retries > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }
        
        // If we got here, all retries failed
        match last_error {
            Some(e) => {
                error!("All connection attempts failed. Last error: {}", e);
                Err(Box::new(e))
            }
            None => {
                error!("All connection attempts failed with unknown error");
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to connect to Neo4j after all retries"
                )))
            }
        }
    }

    async fn init_schema(graph: &Graph) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Initializing Neo4j schema constraints");
        debug!("Using Neo4j 5.x constraint syntax");
        let constraints = vec![
            "CREATE CONSTRAINT unique_metric_id IF NOT EXISTS FOR (n:Metric) REQUIRE n.id IS UNIQUE",
            "CREATE CONSTRAINT unique_system_state_id IF NOT EXISTS FOR (n:SystemState) REQUIRE n.id IS UNIQUE",
            "CREATE CONSTRAINT unique_user_interaction_id IF NOT EXISTS FOR (n:UserInteraction) REQUIRE n.id IS UNIQUE",
            "CREATE CONSTRAINT unique_tool_execution_id IF NOT EXISTS FOR (n:ToolExecution) REQUIRE n.id IS UNIQUE",
            "CREATE CONSTRAINT unique_pattern_id IF NOT EXISTS FOR (n:Pattern) REQUIRE n.id IS UNIQUE",
        ];

        for constraint in constraints {
            let query = Query::new(String::from(constraint));
            debug!("Executing constraint query: {}", constraint);
            let mut result = graph.execute(query).await?;
            // Need to consume the result
            while let Some(_) = result.next().await? {
                // Process each row if needed
            }
            debug!("Successfully created constraint");
        }

        info!("Successfully initialized Neo4j schema");
        Ok(())
    }

    pub async fn store_metric(
        &self,
        metric_type: &str,
        value: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<Node, Box<dyn Error + Send + Sync>> {
        log::debug!("Storing metric of type {} with value {}", metric_type, value);
        let query = Query::new(String::from(
            "CREATE (m:Metric {
                id: randomUUID(),
                type: $type,
                value: $value,
                timestamp: $timestamp
            }) RETURN m"
        ))
        .param("type", metric_type)
        .param("value", value.to_string())
        .param("timestamp", timestamp.to_rfc3339());

        log::debug!("Executing Neo4j query to store metric");
        let mut result = self.graph.execute(query).await?;
        log::debug!("Query executed successfully");

        let row = result.next().await?
            .ok_or_else(|| {
                log::error!("No node was created when storing metric");
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No node created"
                ))
            })?;
        let node = row.get("m")?;
        log::info!("Successfully stored metric node");
        Ok(node)
    }

    pub async fn store_system_state(
        &self,
        state: HashMap<String, serde_json::Value>,
    ) -> Result<Node, Box<dyn Error + Send + Sync>> {
        debug!("Storing system state with {} metrics", state.len());
        let timestamp = Utc::now();
        debug!("Preparing Neo4j query for system state at {}", timestamp);
        
        let state_json = serde_json::to_string(&state)?;
        debug!("System state serialized to JSON (length: {})", state_json.len());
        
        let query = Query::new(String::from(
            "CREATE (s:SystemState {
                id: randomUUID(),
                timestamp: $timestamp,
                state: $state
            }) RETURN s"
        ))
        .param("timestamp", timestamp.to_rfc3339())
        .param("state", state_json);

        debug!("Executing Neo4j query to store system state");
        let mut result = match self.graph.execute(query).await {
            Ok(r) => {
                debug!("Neo4j query executed successfully");
                r
            },
            Err(e) => {
                error!("Failed to execute system state creation query: {}", e);
                return Err(Box::new(e));
            }
        };

        let row = result.next().await?
            .ok_or_else(|| {
                let err = std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No node created"
                );
                log::error!("Failed to create system state node: {}", err);
                Box::new(err)
            })?;
            
        match row.get("s") {
            Ok(node) => {
                log::info!("Successfully stored system state node");
                Ok(node)
            }
            Err(e) => {
                log::error!("Failed to get created system state node from result: {}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn create_relationship(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: RelationType,
        properties: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Relation, Box<dyn Error + Send + Sync>> {
        log::debug!("Creating relationship from {} to {}", from_id, to_id);
        
        let rel_type_str = match rel_type {
            RelationType::Followed => "FOLLOWED",
            RelationType::Caused => "CAUSED",
            RelationType::Related => "RELATED",
            RelationType::Contains => "CONTAINS",
            RelationType::Triggered => "TRIGGERED",
        };
        
        log::debug!("Relationship type: {}", rel_type_str);
        
        log::debug!("Relationship type: {}", rel_type_str);        // Convert properties to a format that Neo4j can understand
        let props: HashMap<String, String> = properties
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect();
            
        log::debug!("Relationship properties: {:?}", props);

        let query_str = format!(
            "MATCH (a), (b)
            WHERE a.id = $from_id AND b.id = $to_id
            CREATE (a)-[r:{}]->(b)
            SET r = $props
            RETURN r",
            rel_type_str
        );
        log::debug!("Built Neo4j query: {}", query_str);

        let query = Query::new(query_str)
            .param("from_id", from_id)
            .param("to_id", to_id)
            .param("props", props);

        log::debug!("Executing Neo4j query to create relationship");
        let mut result = match self.graph.execute(query).await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to execute relationship creation query: {}", e);
                return Err(Box::new(e));
            }
        };

        let row = match result.next().await {
            Ok(Some(r)) => r,
            Ok(None) => {
                let err = std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No relation created"
                );
                log::error!("Failed to create relationship: No relation returned");
                return Err(Box::new(err));
            }
            Err(e) => {
                log::error!("Failed to get next result row: {}", e);
                return Err(Box::new(e));
            }
        };

        match row.get("r") {
            Ok(relation) => {
                log::info!("Successfully created relationship");
                Ok(relation)
            }
            Err(e) => {
                log::error!("Failed to get created relationship from result: {}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn find_patterns(
        &self,
        node_type: ContextNodeType,
        time_window: chrono::Duration,
    ) -> Result<Vec<Node>, Box<dyn Error + Send + Sync>> {
        let node_type_str = match node_type {
            ContextNodeType::Metric => "Metric",
            ContextNodeType::SystemState => "SystemState",
            ContextNodeType::UserInteraction => "UserInteraction",
            ContextNodeType::ToolExecution => "ToolExecution",
            ContextNodeType::Pattern => "Pattern",
        };

        let since = (Utc::now() - time_window).to_rfc3339();
        
        let query_str = format!(
            "MATCH (n:{})
            WHERE n.timestamp >= $since
            WITH n
            ORDER BY n.timestamp
            RETURN n",
            node_type_str
        );
        
        let query = Query::new(query_str)
            .param("since", since);

        let mut result = self.graph.execute(query).await?;
        let mut nodes = Vec::new();
        
        while let Some(row) = result.next().await? {
            nodes.push(row.get("n")?);
        }

        Ok(nodes)
    }
}

// Helper function to get or initialize Neo4j client
pub async fn get_neo4j_context() -> Result<Arc<Neo4jContext>, Box<dyn Error + Send + Sync>> {
    let mut client = NEO4J_CLIENT.lock().await;
    
    if client.is_none() {
        debug!("Initializing new Neo4j connection");
        let url = match std::env::var("NEO4J_URI") {
            Ok(u) => {
                debug!("Using Neo4j URL from environment: {}", u);
                u
            },
            Err(_) => {
                debug!("NEO4J_URI not set, using default: bolt://localhost:7687");
                "bolt://localhost:7687".to_string()
            }
        };
        
        let user = match std::env::var("NEO4J_USER") {
            Ok(u) => {
                debug!("Using Neo4j user from environment: {}", u);
                u
            },
            Err(_) => {
                debug!("NEO4J_USER not set, using default: neo4j");
                "neo4j".to_string()
            }
        };
        
        let password = match std::env::var("NEO4J_PASSWORD") {
            Ok(p) => {
                debug!("Found Neo4j password in environment");
                p
            },
            Err(_) => {
                error!("NEO4J_PASSWORD environment variable is required");
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, 
                    "NEO4J_PASSWORD environment variable is required")));
            }
        };

        debug!("Attempting to connect to Neo4j at {} with user {}", url, user);
        let context = match Neo4jContext::connect(url.clone(), user.clone(), password.clone()).await {
            Ok(ctx) => {
                info!("Successfully created new Neo4j context");
                ctx
            },
            Err(e) => {
                error!("Failed to create Neo4j context: {}", e);
                return Err(e);
            }
        };

        *client = Some(context.graph.clone());
        Ok(Arc::new(context))
    } else {
        debug!("Reusing existing Neo4j connection");
        let graph = client.as_ref().unwrap().clone();
        debug!("Creating Neo4jContext from existing connection");
        Ok(Arc::new(Neo4jContext { graph }))
    }
}