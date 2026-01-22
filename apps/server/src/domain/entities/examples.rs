/// Example demonstrating Pipeline and Job usage
/// This shows how to create a pipeline definition and execute it as a job

#[cfg(test)]
mod example_usage {
    use crate::domain::entities::Pipeline;
    use crate::domain::entities::{Job, PipelineNode};
    use crate::domain::value_objects::{JobStatus, NodeId, NodeName};

    #[test]
    fn example_pipeline_and_job_workflow() {
        // Step 1: Create Pipeline nodes (definition)
        let build_node = PipelineNode::Group {
            id: NodeId::new("build"),
            name: NodeName::new("Build Stage").unwrap(),
            deps: vec![],
        };

        let compile_node = PipelineNode::Action {
            id: NodeId::new("compile"),
            name: NodeName::new("Compile Code").unwrap(),
            deps: vec![NodeId::new("build")],
            command: "cargo".into(),
            args: vec!["build".into(), "--release".into()],
        };

        let test_node = PipelineNode::Action {
            id: NodeId::new("test"),
            name: NodeName::new("Run Tests").unwrap(),
            deps: vec![NodeId::new("compile")],
            command: "cargo".into(),
            args: vec!["test".into()],
        };

        // Step 2: Create Pipeline with validation (no cycles, valid deps, etc.)
        let pipeline =
            Pipeline::create("CI/CD Pipeline", vec![build_node, compile_node, test_node])
                .expect("Failed to create pipeline");

        println!("✓ Pipeline created: {}", pipeline.name());
        println!("  ID: {}", pipeline.id());
        println!("  Nodes: {}", pipeline.nodes().len());

        // Step 3: Create Job from Pipeline (instance of execution)
        let mut job = Job::create_from_pipeline(&pipeline).expect("Failed to create job");

        println!("\n✓ Job created: {}", job.id());
        println!("  Pipeline ID: {}", job.pipeline_id());
        println!("  Initial status: {:?}", job.status());
        println!("  Nodes: {}", job.executions().len());

        // Step 4: Simulate job execution
        println!("\n--- Executing Job ---");

        // Get the build node ID
        let build_id = NodeId::new("build");
        let compile_id = NodeId::new("compile");
        let test_id = NodeId::new("test");

        // Only build is runnable initially (no dependencies)
        let runnable = job.runnable_nodes(&pipeline);
        println!("\n1. Runnable nodes: {}", runnable.len());
        for node_id in &runnable {
            println!("   - {}", node_id);
        }

        // Execute build node
        job.start_node(&build_id)
            .expect("Failed to start build node");
        println!("\n2. Started node: build");
        job.finish_node(&build_id, JobStatus::Completed)
            .expect("Failed to finish build node");
        println!("   Finished with: Completed");

        // Now compile is runnable
        let runnable = job.runnable_nodes(&pipeline);
        println!("\n3. Runnable nodes after build: {}", runnable.len());
        for node_id in &runnable {
            println!("   - {}", node_id);
        }

        // Execute compile node
        job.start_node(&compile_id)
            .expect("Failed to start compile node");
        println!("\n4. Started node: compile");
        job.finish_node(&compile_id, JobStatus::Completed)
            .expect("Failed to finish compile node");
        println!("   Finished with: Completed");

        // Execute test node
        job.start_node(&test_id).expect("Failed to start test node");
        println!("\n5. Started node: test");
        job.finish_node(&test_id, JobStatus::Completed)
            .expect("Failed to finish test node");
        println!("   Finished with: Completed");

        // Job is now complete
        println!("\n✓ Job completed: {:?}", job.status());
        println!("  Created at: {}", job.created_at());
        println!("  Updated at: {}", job.updated_at());

        // Verify all nodes succeeded
        let all_success = job
            .executions()
            .values()
            .all(|exec| exec.state() == JobStatus::Completed);
        println!("  All nodes succeeded: {}", all_success);
    }

    #[test]
    fn example_pipeline_validation() {
        use crate::domain::entities::Pipeline;

        // Try to create pipeline with empty name
        let result = Pipeline::create("", vec![]);
        assert!(result.is_err());
        println!("✓ Empty pipeline name rejected");

        // Try to create pipeline with no nodes
        let result = Pipeline::create("Pipeline", vec![]);
        assert!(result.is_err());
        println!("✓ Empty pipeline nodes rejected");

        // Try to create node with invalid dependency
        let node_with_invalid_dep = PipelineNode::Action {
            id: NodeId::new("action1"),
            name: NodeName::new("Action 1").unwrap(),
            deps: vec![NodeId::new("nonexistent")],
            command: "echo".into(),
            args: vec!["hello".into()],
        };

        let result = Pipeline::create("Pipeline", vec![node_with_invalid_dep]);
        assert!(result.is_err());
        println!("✓ Invalid dependency rejected");
    }
}
