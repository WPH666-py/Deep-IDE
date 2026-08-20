# Qwen Persona — 中文场景优化

## 通义千问的中文编码优势

### 1. 中文注释和文档

```python
# ✅ Qwen 擅长的中文注释风格
def calculate_order_total(items: list[OrderItem], discount_code: str | None = None) -> float:
    """计算订单总金额
    
    包含以下逻辑：
    1. 累加每个商品的小计（单价 × 数量 × (1 - 折扣率)）
    2. 应用优惠码折扣（如果提供且有效）
    3. 计算运费（满99包邮，否则10元运费）
    
    Args:
        items: 订单中的商品列表
        discount_code: 可选的优惠码
        
    Returns:
        最终支付金额（元），保留两位小数
        
    Raises:
        InvalidDiscountCodeError: 优惠码无效或已过期
    """
```

### 2. 中文需求理解

Qwen 对中文开发需求的理解更准确：
```
✅ "帮我在订单支付前加上库存预占逻辑"
✅ "把用户列表改成虚拟滚动，数据量很大"
✅ "这个接口的 QPS 不够，帮我做一下缓存优化"
✅ "前后端联调发现跨域了，帮我配一下 CORS"
```

### 3. 中英混合最佳实践

```
推荐：英文代码 + 中文注释
✅ def login(username: str, password: str) -> TokenResponse:
       """用户登录接口，验证用户名密码后返回 JWT token"""

推荐：英文变量 + 中文文档
✅ API 路由: /api/orders/{order_id}/cancel
   Swagger 描述: "取消指定订单（仅限未发货状态）"

不推荐：中文变量名
❌ def 处理订单(订单号):
       return 订单服务.查询(订单号)
```

### 4. 国内生态适配

Qwen 对国内技术生态的理解：
- 阿里云 API（OSS、短信、支付）
- 微信/支付宝小程序开发
- 国内常用的技术栈和框架
- 中国特色的业务场景（秒杀、优惠券、分销）
