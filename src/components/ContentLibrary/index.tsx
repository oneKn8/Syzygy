import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./ContentLibrary.css";

interface ContentBlock {
  id?: { tb: string; id: { String: string } };
  title: string;
  content: string;
  category: string;
  tags: string[];
  format: string;
  created_at: string;
  updated_at: string;
  usage_count: number;
}

interface Category {
  id?: { tb: string; id: { String: string } };
  name: string;
  description?: string;
  color?: string;
  icon?: string;
}

interface ContentLibraryProps {
  onInsertContent?: (content: string) => void;
  isVisible?: boolean;
}

export function ContentLibrary({ onInsertContent, isVisible = true }: ContentLibraryProps) {
  const [blocks, setBlocks] = useState<ContentBlock[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingBlock, setEditingBlock] = useState<ContentBlock | null>(null);
  const [activeTab, setActiveTab] = useState<"all" | "popular" | "recent">("all");

  // Form state for create/edit
  const [formTitle, setFormTitle] = useState("");
  const [formContent, setFormContent] = useState("");
  const [formCategory, setFormCategory] = useState("");
  const [formTags, setFormTags] = useState("");
  const [formFormat, setFormFormat] = useState("typst");

  const loadBlocks = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      let result: ContentBlock[];

      if (activeTab === "popular") {
        result = await invoke<ContentBlock[]>("get_popular_blocks", { limit: 50 });
      } else if (activeTab === "recent") {
        result = await invoke<ContentBlock[]>("get_recent_blocks", { limit: 50 });
      } else if (searchQuery) {
        result = await invoke<ContentBlock[]>("search_content_blocks", {
          query: searchQuery,
          limit: 50,
        });
      } else if (selectedCategory) {
        result = await invoke<ContentBlock[]>("get_blocks_by_category", {
          category: selectedCategory,
        });
      } else {
        result = await invoke<ContentBlock[]>("list_content_blocks", {
          params: { limit: 100, offset: 0 },
        });
      }

      setBlocks(result);
    } catch (err) {
      console.error("Failed to load content blocks:", err);
      setError(err instanceof Error ? err.message : "Failed to load content");
    } finally {
      setLoading(false);
    }
  }, [activeTab, searchQuery, selectedCategory]);

  const loadCategories = useCallback(async () => {
    try {
      const result = await invoke<Category[]>("list_categories");
      setCategories(result);
    } catch (err) {
      console.error("Failed to load categories:", err);
    }
  }, []);

  useEffect(() => {
    if (isVisible) {
      loadBlocks();
      loadCategories();
    }
  }, [isVisible, loadBlocks, loadCategories]);

  const getBlockId = (block: ContentBlock): string | undefined => {
    if (block.id?.id?.String) {
      return block.id.id.String;
    }
    return undefined;
  };

  const handleInsert = async (block: ContentBlock) => {
    if (onInsertContent) {
      onInsertContent(block.content);

      // Increment usage count
      const id = getBlockId(block);
      if (id) {
        try {
          await invoke("increment_block_usage", { id });
        } catch (err) {
          console.error("Failed to increment usage:", err);
        }
      }
    }
  };

  const handleCreate = async () => {
    try {
      await invoke("create_content_block", {
        input: {
          title: formTitle,
          content: formContent,
          category: formCategory || "Uncategorized",
          tags: formTags.split(",").map((t) => t.trim()).filter(Boolean),
          format: formFormat,
        },
      });
      resetForm();
      setShowCreateModal(false);
      loadBlocks();
    } catch (err) {
      console.error("Failed to create block:", err);
      setError(err instanceof Error ? err.message : "Failed to create block");
    }
  };

  const handleUpdate = async () => {
    if (!editingBlock) return;

    const id = getBlockId(editingBlock);
    if (!id) return;

    try {
      await invoke("update_content_block", {
        id,
        input: {
          title: formTitle,
          content: formContent,
          category: formCategory || "Uncategorized",
          tags: formTags.split(",").map((t) => t.trim()).filter(Boolean),
          format: formFormat,
        },
      });
      resetForm();
      setEditingBlock(null);
      loadBlocks();
    } catch (err) {
      console.error("Failed to update block:", err);
      setError(err instanceof Error ? err.message : "Failed to update block");
    }
  };

  const handleDelete = async (block: ContentBlock) => {
    const id = getBlockId(block);
    if (!id) return;

    if (!confirm(`Delete "${block.title}"?`)) return;

    try {
      await invoke("delete_content_block", { id });
      loadBlocks();
    } catch (err) {
      console.error("Failed to delete block:", err);
      setError(err instanceof Error ? err.message : "Failed to delete block");
    }
  };

  const handleEdit = (block: ContentBlock) => {
    setEditingBlock(block);
    setFormTitle(block.title);
    setFormContent(block.content);
    setFormCategory(block.category);
    setFormTags(block.tags.join(", "));
    setFormFormat(block.format);
  };

  const resetForm = () => {
    setFormTitle("");
    setFormContent("");
    setFormCategory("");
    setFormTags("");
    setFormFormat("typst");
  };

  const handleCreateCategory = async () => {
    const name = prompt("Enter category name:");
    if (!name) return;

    try {
      await invoke("create_category", {
        name,
        description: null,
        color: null,
        icon: null,
      });
      loadCategories();
    } catch (err) {
      console.error("Failed to create category:", err);
    }
  };

  if (!isVisible) return null;

  return (
    <div className="content-library">
      <div className="content-library-header">
        <h3>Content Library</h3>
        <button
          className="btn-create"
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          title="Create new block"
        >
          +
        </button>
      </div>

      <div className="content-library-search">
        <input
          type="text"
          placeholder="Search content..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && loadBlocks()}
        />
        <button onClick={loadBlocks} title="Search">
          Search
        </button>
      </div>

      <div className="content-library-tabs">
        <button
          className={activeTab === "all" ? "active" : ""}
          onClick={() => {
            setActiveTab("all");
            setSelectedCategory(null);
          }}
        >
          All
        </button>
        <button
          className={activeTab === "popular" ? "active" : ""}
          onClick={() => setActiveTab("popular")}
        >
          Popular
        </button>
        <button
          className={activeTab === "recent" ? "active" : ""}
          onClick={() => setActiveTab("recent")}
        >
          Recent
        </button>
      </div>

      <div className="content-library-categories">
        <div className="categories-header">
          <span>Categories</span>
          <button onClick={handleCreateCategory} title="Add category">
            +
          </button>
        </div>
        <div className="categories-list">
          <button
            className={selectedCategory === null && activeTab === "all" ? "active" : ""}
            onClick={() => {
              setSelectedCategory(null);
              setActiveTab("all");
            }}
          >
            All
          </button>
          {categories.map((cat) => (
            <button
              key={cat.name}
              className={selectedCategory === cat.name ? "active" : ""}
              onClick={() => {
                setSelectedCategory(cat.name);
                setActiveTab("all");
              }}
              style={cat.color ? { borderLeftColor: cat.color } : undefined}
            >
              {cat.name}
            </button>
          ))}
        </div>
      </div>

      {error && <div className="content-library-error">{error}</div>}

      <div className="content-library-list">
        {loading ? (
          <div className="content-library-loading">Loading...</div>
        ) : blocks.length === 0 ? (
          <div className="content-library-empty">
            No content blocks found.
            <button onClick={() => setShowCreateModal(true)}>Create your first block</button>
          </div>
        ) : (
          blocks.map((block) => (
            <div key={getBlockId(block) || block.title} className="content-block-item">
              <div className="content-block-header">
                <span className="content-block-title">{block.title}</span>
                <span className="content-block-format">{block.format}</span>
              </div>
              <div className="content-block-preview">
                {block.content.slice(0, 150)}
                {block.content.length > 150 && "..."}
              </div>
              <div className="content-block-meta">
                <span className="content-block-category">{block.category}</span>
                {block.tags.length > 0 && (
                  <span className="content-block-tags">
                    {block.tags.slice(0, 3).join(", ")}
                    {block.tags.length > 3 && `+${block.tags.length - 3}`}
                  </span>
                )}
                <span className="content-block-usage">{block.usage_count} uses</span>
              </div>
              <div className="content-block-actions">
                <button onClick={() => handleInsert(block)} title="Insert into editor">
                  Insert
                </button>
                <button onClick={() => handleEdit(block)} title="Edit block">
                  Edit
                </button>
                <button
                  onClick={() => handleDelete(block)}
                  className="btn-danger"
                  title="Delete block"
                >
                  Delete
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {(showCreateModal || editingBlock) && (
        <div className="content-library-modal-overlay" onClick={() => {
          setShowCreateModal(false);
          setEditingBlock(null);
          resetForm();
        }}>
          <div className="content-library-modal" onClick={(e) => e.stopPropagation()}>
            <h4>{editingBlock ? "Edit Content Block" : "Create Content Block"}</h4>
            <div className="modal-form">
              <label>
                Title
                <input
                  type="text"
                  value={formTitle}
                  onChange={(e) => setFormTitle(e.target.value)}
                  placeholder="Block title"
                />
              </label>
              <label>
                Category
                <select value={formCategory} onChange={(e) => setFormCategory(e.target.value)}>
                  <option value="">Select category...</option>
                  {categories.map((cat) => (
                    <option key={cat.name} value={cat.name}>
                      {cat.name}
                    </option>
                  ))}
                  <option value="Uncategorized">Uncategorized</option>
                </select>
              </label>
              <label>
                Format
                <select value={formFormat} onChange={(e) => setFormFormat(e.target.value)}>
                  <option value="typst">Typst</option>
                  <option value="latex">LaTeX</option>
                  <option value="markdown">Markdown</option>
                  <option value="text">Plain Text</option>
                </select>
              </label>
              <label>
                Tags (comma-separated)
                <input
                  type="text"
                  value={formTags}
                  onChange={(e) => setFormTags(e.target.value)}
                  placeholder="tag1, tag2, tag3"
                />
              </label>
              <label>
                Content
                <textarea
                  value={formContent}
                  onChange={(e) => setFormContent(e.target.value)}
                  placeholder="Enter content..."
                  rows={10}
                />
              </label>
            </div>
            <div className="modal-actions">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  setEditingBlock(null);
                  resetForm();
                }}
              >
                Cancel
              </button>
              <button
                className="btn-primary"
                onClick={editingBlock ? handleUpdate : handleCreate}
                disabled={!formTitle || !formContent}
              >
                {editingBlock ? "Update" : "Create"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ContentLibrary;
