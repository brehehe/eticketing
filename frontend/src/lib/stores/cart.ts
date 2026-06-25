import { writable, derived } from 'svelte/store';
import type { CartItem, Product, ProductVariant } from '$lib/types';

// Core cart items store
const cartItems = writable<CartItem[]>([]);

// Summary derived store — subscribe with $cartSummary
export const cartSummary = derived(cartItems, $items => ({
  subtotal: $items.reduce((s, i) => s + i.subtotal, 0),
  qty:      $items.reduce((s, i) => s + i.qty, 0),
}));

function addItem(product: Product, variant?: ProductVariant, qty = 1) {
  cartItems.update(items => {
    const existing = items.find(i =>
      i.product.id === product.id &&
      (variant ? i.variant?.id === variant.id : !i.variant)
    );
    const price = variant?.price ?? product.price;
    if (existing) {
      return items.map(i => {
        if (i.product.id === product.id &&
            (variant ? i.variant?.id === variant.id : !i.variant)) {
          const newQty = i.qty + qty;
          return { ...i, qty: newQty, subtotal: price * newQty - i.discount };
        }
        return i;
      });
    }
    return [...items, { product, variant, qty, unit_price: price, discount: 0, subtotal: price * qty }];
  });
}

function removeItem(productId: string, variantId?: string) {
  cartItems.update(items => items.filter(i =>
    !(i.product.id === productId && (variantId ? i.variant?.id === variantId : !i.variant))
  ));
}

function updateQty(productId: string, qty: number, variantId?: string) {
  if (qty <= 0) { removeItem(productId, variantId); return; }
  cartItems.update(items => items.map(i => {
    if (i.product.id === productId && (variantId ? i.variant?.id === variantId : !i.variant)) {
      return { ...i, qty, subtotal: i.unit_price * qty - i.discount };
    }
    return i;
  }));
}

function setDiscount(productId: string, discount: number, variantId?: string) {
  cartItems.update(items => items.map(i => {
    if (i.product.id === productId && (variantId ? i.variant?.id === variantId : !i.variant)) {
      return { ...i, discount, subtotal: Math.max(0, i.unit_price * i.qty - discount) };
    }
    return i;
  }));
}

function clear() { cartItems.set([]); }

// cart is the items store + methods — subscribe with $cart to get CartItem[]
export const cart = {
  subscribe: cartItems.subscribe,
  addItem,
  removeItem,
  updateQty,
  setDiscount,
  clear,
};
