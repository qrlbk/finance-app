import { useEffect, useState } from "react";
import { Plus, Pencil, Trash2, Tag, ArrowDownCircle, ArrowUpCircle } from "lucide-react";
import { api, type Category } from "../lib/api";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { EmptyState } from "../components/ui/EmptyState";
import { useToast } from "../components/ui/Toast";

const PRESET_COLORS = [
  "#ef4444", "#f97316", "#eab308", "#22c55e", "#14b8a6",
  "#06b6d4", "#3b82f6", "#6366f1", "#8b5cf6", "#ec4899",
  "#f43f5e", "#64748b",
];

export function Categories() {
  const { showToast } = useToast();
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<number | null>(null);

  const [form, setForm] = useState({
    name: "",
    category_type: "expense" as "income" | "expense",
    color: "#64748b",
    icon: null as string | null,
    parent_id: null as number | null,
  });

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const cats = await api.getCategories();
      setCategories(cats);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const incomeCategories = categories.filter((c) => c.category_type === "income");
  const expenseCategories = categories.filter((c) => c.category_type === "expense");

  const categoryTree = (list: Category[]) => {
    const roots = list.filter((c) => !c.parent_id);
    const childrenOf = (pid: number) => list.filter((c) => c.parent_id === pid);
    return roots.flatMap((root) => [
      { category: root, indent: false },
      ...childrenOf(root.id).map((category) => ({ category, indent: true })),
    ]);
  };
  const incomeTree = categoryTree(incomeCategories);
  const expenseTree = categoryTree(expenseCategories);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return;

    try {
      if (editingId) {
        await api.updateCategory({
          id: editingId,
          name: form.name.trim(),
          color: form.color,
          icon: form.icon,
        });
        showToast("Категория обновлена", "success");
      } else {
        await api.createCategory({
          name: form.name.trim(),
          category_type: form.category_type,
          color: form.color,
          icon: form.icon,
          parent_id: form.parent_id,
        });
        showToast("Категория создана", "success");
      }
      resetForm();
      loadData();
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при сохранении", "error");
    }
  };

  const resetForm = () => {
    setForm({
      name: "",
      category_type: "expense",
      color: "#64748b",
      icon: null,
      parent_id: null,
    });
    setShowForm(false);
    setEditingId(null);
  };

  const handleEdit = (c: Category) => {
    setForm({
      name: c.name,
      category_type: c.category_type as "income" | "expense",
      color: c.color || "#64748b",
      icon: c.icon,
      parent_id: c.parent_id,
    });
    setEditingId(c.id);
    setShowForm(true);
  };

  const handleDeleteConfirm = async () => {
    if (deleteConfirmId === null) return;
    try {
      await api.deleteCategory(deleteConfirmId);
      setDeleteConfirmId(null);
      loadData();
      showToast("Категория удалена", "success");
    } catch (e) {
      setError(String(e));
      showToast(String(e), "error");
    }
  };

  const CategoryCard = ({ category, indent = false }: { category: Category; indent?: boolean }) => (
    <div className={`flex items-center justify-between p-4 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600 transition-colors ${indent ? "ml-6 border-l-2 border-l-zinc-300 dark:border-l-zinc-600" : ""}`}>
      <div className="flex items-center gap-3">
        <div
          className="w-10 h-10 rounded-lg flex items-center justify-center"
          style={{ backgroundColor: `${category.color}20` }}
        >
          <Tag size={18} style={{ color: category.color || "#64748b" }} />
        </div>
        <span className="font-medium">{category.name}</span>
      </div>
      <div className="flex gap-1">
        <button
          onClick={() => handleEdit(category)}
          className="p-1.5 rounded text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700 hover:text-zinc-600 dark:hover:text-white btn-transition"
        >
          <Pencil size={14} />
        </button>
        <button
          onClick={() => setDeleteConfirmId(category.id)}
          className="p-1.5 rounded text-zinc-400 hover:bg-red-500/20 hover:text-red-500 btn-transition"
        >
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 animate-shake">
          {error}
        </div>
      )}

      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium">Категории</h3>
        <button
          onClick={() => {
            setShowForm(true);
            setEditingId(null);
            setForm({
              name: "",
              category_type: "expense",
              color: "#64748b",
              icon: null,
              parent_id: null,
            });
          }}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
        >
          <Plus size={18} />
          Добавить
        </button>
      </div>

      {/* Form */}
      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">
            {editingId ? "Редактировать категорию" : "Новая категория"}
          </h4>

          <div className="grid gap-4 sm:grid-cols-2">
            {!editingId && (
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Тип</label>
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={() => setForm((f) => ({ ...f, category_type: "expense" }))}
                    className={`flex-1 px-4 py-2 rounded-lg border btn-transition ${
                      form.category_type === "expense"
                        ? "bg-red-500/10 border-red-500/30 text-red-500"
                        : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                    }`}
                  >
                    Расход
                  </button>
                  <button
                    type="button"
                    onClick={() => setForm((f) => ({ ...f, category_type: "income" }))}
                    className={`flex-1 px-4 py-2 rounded-lg border btn-transition ${
                      form.category_type === "income"
                        ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-500"
                        : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                    }`}
                  >
                    Доход
                  </button>
                </div>
              </div>
            )}

            {!editingId && (
              <div>
                <label className="block text-sm text-zinc-400 mb-1">Родительская категория</label>
                <select
                  value={form.parent_id ?? ""}
                  onChange={(e) => setForm((f) => ({ ...f, parent_id: e.target.value ? Number(e.target.value) : null }))}
                  className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                >
                  <option value="">— Нет (корневая) —</option>
                  {(form.category_type === "income" ? incomeCategories : expenseCategories)
                    .filter((c) => !c.parent_id)
                    .map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                      </option>
                    ))}
                </select>
              </div>
            )}

            <div className={editingId ? "sm:col-span-2" : ""}>
              <label className="block text-sm text-zinc-400 mb-1">Название</label>
              <input
                type="text"
                value={form.name}
                onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                placeholder="Название категории"
                required
              />
            </div>
          </div>

          <div>
            <label className="block text-sm text-zinc-400 mb-2">Цвет</label>
            <div className="flex flex-wrap gap-2">
              {PRESET_COLORS.map((color) => (
                <button
                  key={color}
                  type="button"
                  onClick={() => setForm((f) => ({ ...f, color }))}
                  className={`w-8 h-8 rounded-lg transition-transform hover:scale-110 ${
                    form.color === color ? "ring-2 ring-offset-2 ring-offset-white dark:ring-offset-zinc-900 ring-zinc-400" : ""
                  }`}
                  style={{ backgroundColor: color }}
                />
              ))}
              <input
                type="color"
                value={form.color}
                onChange={(e) => setForm((f) => ({ ...f, color: e.target.value }))}
                className="w-8 h-8 rounded-lg cursor-pointer"
                title="Выбрать другой цвет"
              />
            </div>
          </div>

          <div className="flex gap-2">
            <button
              type="submit"
              className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              {editingId ? "Сохранить" : "Добавить"}
            </button>
            <button
              type="button"
              onClick={resetForm}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              Отмена
            </button>
          </div>
        </form>
      )}

      {/* Expense Categories */}
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <ArrowDownCircle size={18} className="text-red-500" />
          <h4 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">
            Расходы ({expenseCategories.length})
          </h4>
        </div>
        {loading ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="p-4 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
                  <div className="h-5 w-24 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
                </div>
              </div>
            ))}
          </div>
        ) : expenseCategories.length > 0 ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {expenseTree.map(({ category, indent }) => (
              <CategoryCard key={category.id} category={category} indent={indent} />
            ))}
          </div>
        ) : (
          <p className="text-sm text-zinc-400">Нет категорий расходов</p>
        )}
      </div>

      {/* Income Categories */}
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <ArrowUpCircle size={18} className="text-emerald-500" />
          <h4 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">
            Доходы ({incomeCategories.length})
          </h4>
        </div>
        {loading ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className="p-4 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
                  <div className="h-5 w-24 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
                </div>
              </div>
            ))}
          </div>
        ) : incomeCategories.length > 0 ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {incomeTree.map(({ category, indent }) => (
              <CategoryCard key={category.id} category={category} indent={indent} />
            ))}
          </div>
        ) : (
          <p className="text-sm text-zinc-400">Нет категорий доходов</p>
        )}
      </div>

      {!loading && categories.length === 0 && !showForm && (
        <EmptyState
          icon={Tag}
          title="Нет категорий"
          description="Создайте категории для классификации ваших доходов и расходов."
          action={
            <button
              onClick={() => {
                setShowForm(true);
                setEditingId(null);
              }}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              <Plus size={18} />
              Добавить категорию
            </button>
          }
        />
      )}

      <ConfirmDialog
        open={deleteConfirmId !== null}
        title="Удалить категорию?"
        message="Категорию можно удалить только если она не используется в транзакциях или бюджетах."
        confirmLabel="Удалить"
        variant="danger"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeleteConfirmId(null)}
      />
    </div>
  );
}
