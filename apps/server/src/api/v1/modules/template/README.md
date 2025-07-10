# Module Template

This template provides a starting point for creating new modules in the application. It follows the same structure and patterns as the existing modules, such as the user and teams modules.

## How to Use This Template

1. **Copy the Template**: Copy the entire `template` directory to create a new module.
   ```bash
   cp -r apps/server/src/api/v1/modules/template apps/server/src/api/v1/modules/your_module_name
   ```

2. **Rename Entity**: Replace all occurrences of "Entity" with your entity name in all files.
   - For example, replace "Entity" with "Product", "Customer", etc.
   - This includes struct names, trait names, function names, and comments.

3. **Update Table References**: Replace all occurrences of "entities" with your actual database table name.
   - Look for comments like `// Replace with your actual table` to find these occurrences.

4. **Customize Fields**: Update the fields in the DTOs and models to match your entity's requirements.
   - Add, remove, or modify fields in `NewEntity`, `EntityResponse`, `UpdateEntityRequest`, etc.
   - Update the validation rules as needed.

5. **Update Imports**: Update the imports to reference your new module and models.
   - Replace `crate::api::v1::modules::template` with `crate::api::v1::modules::your_module_name`.
   - Update the model import to reference your actual model.

6. **Register the Module**: Add your new module to the API router in the appropriate place.

## Module Structure

The template follows a standard structure:

- **mod.rs**: Exports the module's components.
- **controller.rs**: Handles HTTP requests and responses.
- **dto.rs**: Defines Data Transfer Objects for requests and responses.
- **repository.rs**: Handles database operations. Uses the `#[derive(Repository)]` macro to automatically implement the Repository trait.
- **service.rs**: Contains business logic.

## Example Customization

Here's an example of how to customize the template for a "Product" module:

1. Copy the template:
   ```bash
   cp -r apps/server/src/api/v1/modules/template apps/server/src/api/v1/modules/product
   ```

2. In each file, replace:
   - "Entity" with "Product"
   - "entity" with "product"
   - "entities" with "products"

3. Update the fields in dto.rs to match your product entity:
   ```rust
   pub struct NewProduct {
       pub name: String,
       pub price: f64,
       pub description: String,
       pub category_id: uuid::Uuid,
   }
   ```

4. Update the imports to reference your new module:
   ```rust
   use crate::api::v1::modules::product::dto::{NewProductRequest, UpdateProductRequest};
   ```

5. Register your new module in the API router.

## Best Practices

- Keep the same structure and patterns as the existing modules.
- Use meaningful names for your entities and fields.
- Add appropriate validation rules for your fields.
- Document your code with comments.
- Write tests for your new module.
